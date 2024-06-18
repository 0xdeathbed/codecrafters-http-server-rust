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
    let status_line = match request_line[1] {
        "/" => "HTTP/1.1 200 OK",
        _ => "HTTP/1.1 404 Not Found",
    };

    let mut headers = Vec::new();
    while let Some(line) = lines.next_line().await? {
        if line.len() == 0 {
            break;
        }

        headers.push(line);
    }

    let response = format!("{status_line}\r\n\r\n");
    let response = Bytes::from(response);

    stream.write_all(&response).await?;

    Ok(())
}
