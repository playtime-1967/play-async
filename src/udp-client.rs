#![allow(warnings)]
use std::env;
use std::error::Error;
use std::io::{stdin, Read};
use std::net::SocketAddr;
use tokio::net::UdpSocket;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let remote_addr: SocketAddr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:8080".into())
        .parse()?;

    // We use port 0 to let the OS allocate an available port for us.
    let local_addr: SocketAddr = if remote_addr.is_ipv4() {
        "0.0.0.0:0"
    } else {
        "[::]:0"
    }
    .parse()?;

    let socket = UdpSocket::bind(local_addr).await?;

    socket.connect(&remote_addr).await?;
    let data = get_stdin_data()?;
    socket.send(&data).await?;

    //UDP datagrams have a theoretical maximum size of 65,535 bytes, including headers:
    // Header sizes: 8 bytes for the UDP header. 20 bytes for the IPv4 header (or more for IPv6).
    // Payload size: 65,535 - 8 - 20 = 65,507 bytes.
    const MAX_DATAGRAM_SIZE: usize = 65_507; //equivalent to 65535

    let mut data = vec![0u8; MAX_DATAGRAM_SIZE];
    let len = socket.recv(&mut data).await?;
    println!(
        "Received {} bytes:\n{}",
        len,
        String::from_utf8_lossy(&data[..len])
    );

    Ok(())
}

fn get_stdin_data() -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut buf = Vec::new();
    stdin().read_to_end(&mut buf)?;
    Ok(buf)
}

//How to run
//- terminal one:cargo run --bin echo-udp
// terminal two: cargo run --bin udp-client or cargo run --bin udp-client -- 127.0.0.1:8080
//type something in the udp-client terminal, and EOF(CTRL+Z)
