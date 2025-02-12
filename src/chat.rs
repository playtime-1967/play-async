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
        //to track the lifecycle of spawned tasks on the Tokio runtime.
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

//the two ends(sender and receive) of the communication channel
type Sx = mpsc::UnboundedSender<String>;
type Rx = mpsc::UnboundedReceiver<String>;

struct Shared {
    peers: HashMap<SocketAddr, Sx>,
}
impl Shared {
    fn new() -> Self {
        Shared {
            peers: HashMap::new(),
        }
    }

    //send a `LineCodec` encoded message to every peer, except for the sender.
    async fn broadcast(&mut self, sender: SocketAddr, message: &str) {
        for peer in self.peers.iter_mut() {
            if *peer.0 != sender {
                let _ = peer.1.send(message.into());
            }
        }
    }
}

//Peer reflects the roles of a client.
struct Peer {
    //handle the Peer's socket.
    framed_stream: Framed<TcpStream, LinesCodec>,
    //receive messages from peers.
    rx: Rx,
}
impl Peer {
    async fn new(
        state: Arc<Mutex<Shared>>,
        framed_stream: Framed<TcpStream, LinesCodec>,
    ) -> io::Result<Peer> {
        let addr = framed_stream.get_ref().peer_addr()?;
        //create a channel for the peer. Unbounded channels allow infinite messages but can grow in memory. Bounded channels have a fixed capacity, providing backpressure.
        let (sx, rx) = mpsc::unbounded_channel();

        state.lock().await.peers.insert(addr, sx);

        Ok(Peer { framed_stream, rx })
    }
}

//process an individual chat peer
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
            //didn't get a line so return early here.
            tracing::error!("Failed to get username from {}. Peer disconnected.", addr);
            return Ok(());
        }
    };

    //register the peer with state which internally sets up some channels.
    let mut peer = Peer::new(state.clone(), framed_stream).await?;

    //a peer has connected, let's let everyone know.
    //MutexGuard 'state.lock()' is dropped as soon as it's no longer needed.
    {
        let mut state = state.lock().await;
        let msg = format!("{username} has joined the chat");
        tracing::info!("{}", msg);
        state.broadcast(addr, &msg).await;
    }

    //process incoming messages until the stream is exhausted by a disconnect.
    loop {
        //any unfinished tasks from the previous iteration remain active and are checked again in the next iteration.
        tokio::select! {
            //a message was received from a peer. Send it to the current peer.
            Some(msg) = peer.rx.recv() => {
                peer.framed_stream.send(&msg).await?;
            }
            result = peer.framed_stream.next() => match result {
                Some(Ok(msg)) => {
                    //a message was received from the current peer. Broadcast it to the other peers.
                    let mut state = state.lock().await;
                    let msg = format!("{username}: {msg}");
                    tracing::info!("{}", msg);
                    state.broadcast(addr, &msg).await;
                }
                Some(Err(e)) => {
                    tracing::error!(
                        "an error occurred while processing messages for {}; error = {:?}",
                        username,
                        e
                    );
                }
                //the stream has been exhausted.
                None => break,
            },
        }
    }

    //the peer disconnected. Let everyone still connected know about it.
    {
        let mut state = state.lock().await;
        state.peers.remove(&addr);

        let msg = format!("{username} has left the chat");
        tracing::info!("{}", msg);
        state.broadcast(addr, &msg).await;
    }

    Ok(())
}
