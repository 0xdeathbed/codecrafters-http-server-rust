use anyhow::Result;
use bytes::Bytes;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::{TcpListener, TcpStream},
};

#[tokio::main]
async fn main() -> Result<()> {
    println!("Logs from your program will appear here!");

    let listener = TcpListener::bind("127.0.0.1:4221").await?;

    loop {
        match listener.accept().await {
            Ok((stream, _socketaddr)) => {
                println!("accepted new connection");
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
    let request_line = buf_reader.lines().next_line().await?.unwrap();

    let status_line = match request_line.as_str() {
        "GET / HTTP/1.1" => "HTTP/1.1 200 OK\r\n\r\n",
        _ => "",
    };

    let response = Bytes::from(status_line);

    stream.write_all(&response).await?;

    Ok(())
}
