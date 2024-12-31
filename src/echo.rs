// a server which will create a TCP listener, accept connections in a loop, and write back everything that's read off of each TCP connection.
// Because the Tokio runtime uses a thread pool, each TCP connection is processed concurrently with all other TCP connections across multiple threads.

#![allow(warnings)]
use futures::future::err;
use std::env;
use std::error::Error;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // passing an address to listen, otherwise we'll just set up our TCP listener on 127.0.0.1:8080.
    let addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:8080".to_string());

    // Create a TCP listener, is bound to the address... Must be associated with an event loop.
    let listener = TcpListener::bind(&addr).await?;
    println!("Listening on: {addr}");

    loop {
        // Asynchronously wait for an inbound socket.
        let (mut socket, _) = listener.accept().await?; //creates a new socket for each connection

        //Executing a new task to run concurrently, rather than blocking one on completion of another.
        //The task stay alive as long as:1- The server is running. 2- The client connection is active.
        tokio::spawn(async move {
            //async move: force the async block to take ownership of `socket` (Or any other referenced variables).

            //Network sockets operate on raw bytes. A u8 (8-bit unsigned integer) represents a single byte.
            let mut buf = vec![0; 1024]; //1024 bytes (1 KB) a common default buffer size that balances Efficiency and Memory Use

            // In a loop, read data from the socket and write the data back.
            loop {
                let n = socket
                    //socket.read() is event-driven, The task "sleeps" while waiting for a message.
                    .read(&mut buf) //read up to 1024 bytes- is event-driven. The task "sleeps" while waiting for a message.
                    .await
                    .expect("failed to read data from socket");

                if n == 0 {
                    return;
                }
                println!(
                    "received: {:?}",
                    String::from_utf8(buf[0..n].to_vec()).unwrap() //print the actual string rather than raw bytes
                );
                socket
                    .write_all(&buf[0..n])
                    .await
                    .expect("failed to write data to socket");
            }
        });
    }
}

//1- cargo run --bin echo
//2- cargo run --bin connect 127.0.0.1:8080
//type something in connect terminal