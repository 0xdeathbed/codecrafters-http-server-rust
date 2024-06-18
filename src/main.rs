use std::collections::HashMap;

use anyhow::Result;
use bytes::Bytes;
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

enum HttpStatus {
    Ok,
    NotFound,
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
    let mut response_headers = String::new();
    let mut response_body = String::new();
    let status = match request_line[1] {
        "/" => HttpStatus::Ok,
        "/user-agent" => {
            if let Some(user_agent) = req_headers.get("User-Agent") {
                response_headers.push_str(&headers_based_on_body(&user_agent));
                response_body.push_str(user_agent);
            }

            HttpStatus::Ok
        }
        echo if echo.starts_with("/echo") => {
            let echo = echo.replace("/echo/", "");
            response_headers.push_str(&headers_based_on_body(&echo));

            response_body.push_str(&echo);

            HttpStatus::Ok
        }
        _ => HttpStatus::NotFound,
    };

    let status_line = match status {
        HttpStatus::Ok => "200 OK",
        HttpStatus::NotFound => "404 Not Found",
    };

    let response = format!("HTTP/1.1 {status_line}\r\n{response_headers}\r\n{response_body}");
    let response = Bytes::from(response);

    stream.write_all(&response).await?;

    Ok(())
}

fn headers_based_on_body(body: &str) -> String {
    format!(
        "Content-Type: text/plain\r\nContent-Length: {}\r\n",
        body.len()
    )
}
