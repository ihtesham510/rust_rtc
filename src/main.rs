use std::{collections::HashMap, sync::Arc};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    sync::Mutex,
};
use tokio_tungstenite::WebSocketStream;

type Connections = Arc<Mutex<HashMap<String, WebSocketStream<tokio::net::TcpStream>>>>;
type Rooms = Arc<Mutex<HashMap<String, Vec<String>>>>;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let tcp = tokio::net::TcpListener::bind("127.0.0.1:8080")
        .await
        .unwrap();

    println!("Server running on localhost:8080");

    while let Ok((stream, addr)) = tcp.accept().await {
        tokio::spawn(handle_res(stream, addr));
    }
    return Ok(());
}

async fn handle_res(mut stream: tokio::net::TcpStream, addr: std::net::SocketAddr) {
    let mut buf = [0, 255];
    match stream.read(&mut buf).await {
        Ok(0) => return,
        Ok(n) => {
            let received = String::from_utf8_lossy(&buf[..n]);
            println!("received from {}:{}", addr, received);
            let res = "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\n\r\nHello World!";
            stream.write_all(res.as_bytes()).await.unwrap();
        }
        Err(err) => eprintln!("Error while reading stream :{:?}", err),
    }
}
