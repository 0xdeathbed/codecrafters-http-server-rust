mod http;

use anyhow::Result;
use http::{HttpResponseBuilder, HttpStatus};
use std::{collections::HashMap, env, path::PathBuf};
use tokio::{
    fs::File,
    io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader},
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
    let buf_reader = BufReader::new(&mut stream);
    let mut lines = buf_reader.lines();

    let request_line = lines.next_line().await?.unwrap();
    let request_line: Vec<&str> = request_line.split_whitespace().collect();

    let mut req_headers: HashMap<String, String> = HashMap::new();
    while let Some(line) = lines.next_line().await? {
        if line.len() == 0 {
            break;
        }

        let kv_pair: Vec<&str> = line
            .split(':')
            .collect::<Vec<&str>>()
            .iter()
            .map(|s| s.trim())
            .collect();

        req_headers.insert(kv_pair[0].to_string(), kv_pair[1].to_string());
    }

    let mut response_http = HttpResponseBuilder::new();
    match request_line[1] {
        "/" => response_http.add_status(HttpStatus::Ok),
        "/user-agent" => {
            if let Some(user_agent) = req_headers.get("User-Agent") {
                response_http.add_body_with_req_headers(&user_agent, "text/plain");
            }

            response_http.add_status(HttpStatus::Ok)
        }
        echo if echo.starts_with("/echo") => {
            let echo = echo.replace("/echo/", "");
            response_http.add_body_with_req_headers(&echo, "text/plain");

            response_http.add_status(HttpStatus::Ok)
        }
        files if files.starts_with("/files/") => {
            let filename = files.replace("/files/", "");
            let mut directory = "/tmp/".to_string();
            let mut args = env::args();
            args.next();
            if let Some(arg) = args.next() {
                if arg == "--directory" {
                    directory = args.next().unwrap();
                }
            }

            let file_path = PathBuf::from(format!("{directory}/{filename}"));
            if file_path.exists() {
                let mut file = File::open(file_path).await?;
                let mut file_content = String::new();
                file.read_to_string(&mut file_content).await?;

                response_http.add_body_with_req_headers(&file_content, "application/octet-stream");
                response_http.add_status(HttpStatus::Ok);
            } else {
                response_http.add_status(HttpStatus::NotFound)
            }
        }
        _ => response_http.add_status(HttpStatus::NotFound),
    };

    let response = response_http.build().convert_to_bytes();

    stream.write_all(&response).await?;

    Ok(())
}
