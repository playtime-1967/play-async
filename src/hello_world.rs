// A simple client that opens a TCP stream, writes "hello world\n", and closes the connection.
#![allow(warnings)]
use std::error::Error;
use tokio::io::AsyncWriteExt;
use tokio::net::{TcpListener, TcpStream, UdpSocket};

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn Error>> {
    // Open a TCP stream to the socket address.
    //
    // Note that this is the Tokio TcpStream, which is fully async.
    let mut stream = TcpStream::connect("127.0.0.1:6142").await?;
    println!("created stream");

    let result = stream.write_all(b"hello world\n").await;
    println!("wrote to stream; success={:?}", result.is_ok());

    Ok(())
}

//1- run the powershell script as a server.
//C:\TMM\github\sample\tcp_listener
//wait till it's listening
// 2- run the app
//cargo run --bin hw
//client will send msg to the server.
