mod http;

use anyhow::Result;
use http::{HttpResponseBuilder, HttpStatus};
use std::collections::HashMap;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
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
                response_http.add_body_with_req_headers(&user_agent);
            }
        }
        echo if echo.starts_with("/echo") => {
            let echo = echo.replace("/echo/", "");
            response_http.add_body_with_req_headers(&echo);
        }
        _ => response_http.add_status(HttpStatus::NotFound),
    };

    let response = response_http.build().convert_to_bytes();

    stream.write_all(&response).await?;

    Ok(())
}
