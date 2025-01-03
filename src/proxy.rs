use futures::FutureExt;
use std::env;
use std::error::Error;
use tokio::io::copy_bidirectional;
use tokio::net::{TcpListener, TcpStream};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let listen_addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:8081".to_string());

    let server_addr = env::args()
        .nth(2)
        .unwrap_or_else(|| "127.0.0.1:8080".to_string());

    println!("Listening on: {listen_addr}");
    println!("Proxying to: {server_addr}");

    let listener = TcpListener::bind(listen_addr).await?;
    while let Ok((mut inbound, _)) = listener.accept().await {
        let mut outbound = TcpStream::connect(server_addr.clone()).await?;

        tokio::spawn(async move {
            //copy_bidirectional is responsible for transferring data in both directions between two streams using internal buffers.
            //There’s no mechanism in copy_bidirectional to intercept the data being transferred between the two streams. The function doesn’t provide hooks or callbacks to inspect the data during transfer.
            copy_bidirectional(&mut inbound, &mut outbound)
                .map(|r| {
                    if let Err(err) = r {
                        println!("Failed to transfer; error={err}");
                    }
                })
                .await;
        });
    }

    Ok(())
}

//How to run
//1-  cargo run --bin proxy
//2-  cargo run --bin echo
//3-  cargo run --bin connect 127.0.0.1:8081
// type something in the connect terminal.
