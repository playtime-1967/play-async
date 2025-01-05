use futures::SinkExt;
use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::io;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, Mutex};
use tokio_stream::StreamExt;
use tokio_util::codec::{Framed, LinesCodec};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    use tracing_subscriber::{fmt::format::FmtSpan, EnvFilter};
    std::env::set_var("RUST_LOG", "tokio=trace"); //to enable additional traces emitted by Tokio itself
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("chat=info".parse()?))
        // to track the lifecycle of spawned tasks on the Tokio runtime.
        .with_span_events(FmtSpan::FULL)
        .init();

    let addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:6142".to_string());

    let listener = TcpListener::bind(&addr).await?;
    tracing::info!("server running on {}", addr);

    let state = Arc::new(Mutex::new(Shared::new()));
    loop {
        let (stream, addr) = listener.accept().await?;

        let state = Arc::clone(&state);

        tokio::spawn(async move {
            if let Err(e) = process(state, stream, addr).await {
                tracing::info!("an error occurred; error = {:?}", e);
            }
        });
    }
}
//------------------------------------------------------------------------------

//The two ends(sender and receive) of the communication channel
type Sx = mpsc::UnboundedSender<String>;
type Rx = mpsc::UnboundedReceiver<String>;

//Shared reflects the roles of the chat server.
struct Shared {
    peers: HashMap<SocketAddr, Sx>,
}
impl Shared {
    fn new() -> Self {
        Shared {
            peers: HashMap::new(),
        }
    }

    // Send a `LineCodec` encoded message to every peer, except for the sender.
    async fn broadcast(&mut self, sender: SocketAddr, message: &str) {
        for peer in self.peers.iter_mut() {
            if *peer.0 != sender {
                let _ = peer.1.send(message.into());
            }
        }
    }
}

//--------------------------------------------------------------------------------
//Peer reflects the roles of a client.
struct Peer {
    // handle the socket.
    framed_stream: Framed<TcpStream, LinesCodec>,
    // receive messages from peers.
    rx: Rx,
}
impl Peer {
    async fn new(
        state: Arc<Mutex<Shared>>,
        framed_stream: Framed<TcpStream, LinesCodec>,
    ) -> io::Result<Peer> //A specialized [Result] type for I/O operations
    {
        // Get the client socket address
        let addr = framed_stream.get_ref().peer_addr()?;
        // Create a channel for this peer
        //Unbounded channels allow infinite messages but can grow in memory. Bounded channels have a fixed capacity, providing backpressure.
        let (sx, rx) = mpsc::unbounded_channel();

        // Add an entry for this `Peer` in the shared state map.
        state.lock().await.peers.insert(addr, sx);

        Ok(Peer { framed_stream, rx })
    }
}

// Process an individual chat client
async fn process(
    state: Arc<Mutex<Shared>>,
    stream: TcpStream,
    addr: SocketAddr,
) -> Result<(), Box<dyn Error>> {
    let mut framed_stream = Framed::new(stream, LinesCodec::new());

    framed_stream.send("Please enter your username:").await?;

    let username = match framed_stream.next().await {
        Some(Ok(line)) => line,
        _ => {
            // We didn't get a line so we return early here.
            tracing::error!("Failed to get username from {}. Client disconnected.", addr);
            return Ok(());
        }
    };

    // Register our peer with state which internally sets up some channels.
    let mut peer = Peer::new(state.clone(), framed_stream).await?;

    // A client has connected, let's let everyone know.
    //This block ensures the MutexGuard (acquired via state.lock().await) is dropped as soon as it's no longer needed.
    {
        let mut state = state.lock().await;
        let msg = format!("{username} has joined the chat");
        tracing::info!("{}", msg);
        state.broadcast(addr, &msg).await;
    }

    // Process incoming messages until our stream is exhausted by a disconnect.
    loop {
        //Each iteration of the loop re-invokes tokio::select!, re-subscribing to the futures (e.g., peer.rx.recv() and peer.framed_stream.next()).
        //Any unfinished tasks from the previous iteration remain active and are checked again in the next iteration.
        tokio::select! {
            // A message was received from a peer. Send it to the current peer.
            Some(msg) = peer.rx.recv() => {
                peer.framed_stream.send(&msg).await?;
            }
            result = peer.framed_stream.next() => match result {
                Some(Ok(msg)) => {
                    // A message was received from the current peer, we should broadcast this message to the other peers.
                    let mut state = state.lock().await;
                    let msg = format!("{username}: {msg}");

                    state.broadcast(addr, &msg).await;
                }
                // An error occurred.
                Some(Err(e)) => {
                    tracing::error!(
                        "an error occurred while processing messages for {}; error = {:?}",
                        username,
                        e
                    );
                }
                // The stream has been exhausted.
                None => break,
            },
        }
    }

    //Means the client was disconnected! Let's let everyone still connected know about it.
    {
        let mut state = state.lock().await;
        state.peers.remove(&addr);

        let msg = format!("{username} has left the chat");
        tracing::info!("{}", msg);
        state.broadcast(addr, &msg).await;
    }

    Ok(())
}

//How to run- 3 or more terminals
//1- cargo run --bin chat
//2- telnet localhost 6142
//3- telnet localhost 6142
