use std::error::Error;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream, UdpSocket};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Hello, play-async!");

    Ok(())
}
