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

async fn handle_http_response(mut stream: TcpStream) -> Result<()> {
    let buf_reader = BufReader::new(&mut stream);
    let mut lines = buf_reader.lines();

    let request_line = lines.next_line().await?.unwrap();
    let request_line: Vec<&str> = request_line.split_whitespace().collect();

    let mut response_headers = String::new();
    let mut response_body = String::new();
    let status_line = match request_line[1] {
        "/" => "200 OK",
        echo if echo.starts_with("/echo") => {
            let echo = echo.replace("/echo/", "");
            response_headers.push_str(&format!("Content-Type: text/plain\r\n"));
            response_headers.push_str(&format!("Content-Length: {}\r\n", echo.len()));

            response_body.push_str(&echo);

            "200 OK"
        }
        _ => "404 Not Found",
    };

    let mut headers = Vec::new();
    while let Some(line) = lines.next_line().await? {
        if line.len() == 0 {
            break;
        }

        headers.push(line);
    }

    let response = format!("HTTP/1.1 {status_line}\r\n{response_headers}\r\n{response_body}");
    let response = Bytes::from(response);

    stream.write_all(&response).await?;

    Ok(())
}
