// An UDP echo server that just sends back everything that it receives.

#![allow(warnings)]
use std::error::Error;
use std::net::SocketAddr;
use std::{env, io};
use tokio::net::UdpSocket;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:8080".to_string());

    let socket = UdpSocket::bind(&addr).await?;
    println!("Listening on: {}", socket.local_addr()?);

    let server = UDPServer {
        socket,
        buf: vec![0; 1024],
        datagram_message: None,
    };

    // This starts the server task.
    server.run().await?;

    Ok(())
}

struct UDPServer {
    socket: UdpSocket,
    buf: Vec<u8>,
    //A datagram message refers to a single, self-contained unit of data sent over a network using a connectionless communication protocol like UDP.
    //Unlike a stream of data (in TCP), a datagram is discrete and independent, carrying all the information needed to deliver it from sender to receiver.
    //Connectionless:Unlike TCP, no persistent connection is established between the sender and receiver.
    //no guarantees about delivery, order, or reliability.
    datagram_message: Option<(usize, SocketAddr)>,
}

impl UDPServer {
    pub async fn run(self) -> Result<(), io::Error> {
        let UDPServer {
            socket,
            mut buf,
            mut datagram_message,
        } = self;

        loop {
            if let Some((size, peer)) = datagram_message {
                let amt = socket.send_to(&buf, peer).await?;
                println!("Echoed {amt}/{size} bytes to {peer}");
            }

            datagram_message = Some(socket.recv_from(&mut buf).await?);
        }
    }
}

//How to run
//cargo run --bin echo-udp
//in another terminal:
//cargo run --bin connect -- --udp 127.0.0.1:8080
//type something in connect terminal
