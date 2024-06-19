mod http;

use anyhow::Result;
use http::{HttpResponseBuilder, HttpStatus};
use itertools::Itertools;
use std::{collections::HashMap, env, path::PathBuf};
use tokio::{
    fs::{File, OpenOptions},
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};

#[tokio::main]
async fn main() -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:4221").await?;
    println!(
        "Server Started listening on {}",
        listener.local_addr()?.to_string()
    );
    loop {
        match listener.accept().await {
            Ok((stream, socketaddr)) => {
                println!("{} Connected", socketaddr.to_string());
                handle_http_response(stream).await?;
            }
            Err(e) => {
                println!("error: {}", e);
                break;
            }
        }
    }

    Ok(())
}

async fn handle_http_response(mut stream: TcpStream) -> Result<()> {
    let mut buffer = [0; 512];
    stream.read(&mut buffer).await?;

    let raw_request = String::from_utf8_lossy(&buffer);

    let raw_request_parts = raw_request.split("\r\n\r\n").collect_vec();

    let request_line_headers = raw_request_parts[0].split("\r\n").collect_vec();

    let req_body = raw_request_parts[1].trim().replace("\0", "");
    let request_line = request_line_headers[0].split_whitespace().collect_vec();

    let mut req_headers: HashMap<String, String> = HashMap::new();
    for &header in request_line_headers[1..].iter() {
        let kv_pair = header
            .split(':')
            .collect_vec()
            .iter()
            .map(|s| s.trim())
            .collect_vec();

        req_headers.insert(kv_pair[0].to_string(), kv_pair[1].to_string());
    }

    let mut directory = "/tmp/".to_string();
    let mut args = env::args();
    args.next();
    if let Some(arg) = args.next() {
        if arg == "--directory" {
            directory = args.next().unwrap();
        }
    }

    let mut response_http = HttpResponseBuilder::new();

    let compression = req_headers.get("Accept-Encoding");
    if let Some(compression_scheme) = compression {
        response_http.enable_compression(&compression_scheme);
    }

    let media_type = if let Some(media_type) = req_headers.get("Content-Type") {
        media_type
    } else {
        "text/plain"
    };
    match request_line[0] {
        "GET" => match request_line[1] {
            "/" => response_http.add_status(HttpStatus::Ok),
            "/user-agent" => {
                if let Some(user_agent) = req_headers.get("User-Agent") {
                    response_http.add_body_with_req_headers(&user_agent, "text/plain");
                }

                response_http.add_status(HttpStatus::Ok)
            }
            echo if echo.starts_with("/echo") => {
                let echo = echo.replace("/echo/", "");

                response_http.add_body_with_req_headers(&echo, media_type);

                response_http.add_status(HttpStatus::Ok)
            }
            files if files.starts_with("/files/") => {
                let filename = files.replace("/files/", "");

                let file_path = PathBuf::from(format!("{directory}/{filename}"));
                if file_path.exists() {
                    let mut file = File::open(file_path).await?;
                    let mut file_content = String::new();
                    file.read_to_string(&mut file_content).await?;

                    response_http
                        .add_body_with_req_headers(&file_content, "application/octet-stream");
                    response_http.add_status(HttpStatus::Ok);
                } else {
                    response_http.add_status(HttpStatus::NotFound)
                }
            }
            _ => response_http.add_status(HttpStatus::NotFound),
        },
        "POST" => match request_line[1] {
            files if files.starts_with("/files/") => {
                let filename = files.replace("/files/", "");
                let file_path = PathBuf::from(format!("{directory}/{filename}"));

                if media_type == "application/octet-stream" {
                    let mut file = OpenOptions::new()
                        .write(true)
                        .truncate(true)
                        .create(true)
                        .open(file_path)
                        .await?;

                    file.write(req_body.as_bytes()).await?;

                    response_http.add_status(HttpStatus::Created);
                } else {
                    response_http.add_status(HttpStatus::InternalServerError);
                }
            }
            _ => response_http.add_status(HttpStatus::NotFound),
        },
        _ => response_http.add_status(HttpStatus::NotImplemented),
    }

    let response = response_http.build().await?;

    stream.write_all(&response).await?;

    Ok(())
}
