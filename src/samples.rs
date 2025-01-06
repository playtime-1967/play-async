use mini_redis::client;
use std::error::Error;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream, UdpSocket};
use tokio::sync::mpsc;
use tokio::sync::oneshot;
use tokio::time::{sleep, Duration};
use tokio::{spawn, stream};
use tokio_stream::StreamExt;
#[tokio::main]
async fn main() {
    //-----------------------------------------------------Streams and Adapters
    let data = &[1, 2, 3];
    let mut stream = tokio_stream::iter(data);

    while let Some(v) = stream.next().await {
        println!("GOT = {:?}", v);
    }

    let mut taked_items = tokio_stream::iter(data).skip(2).take(1);
    while let Some(v) = taked_items.next().await {
        println!("GOT = {:?}", v);
    }

    //--------------------------------------------------------- Task Cancellation

    let (tx, rx) = oneshot::channel();

    //When the spawned task begins, it runs the tokio::select! block
    let task = spawn(async move {
        tokio::select! {
        last= async_work() =>{ //The => separates the future (or async operation) from the block of code that executes when the future completes
            println!("Work completed!- last: {last}");

        }
        _= rx =>{
            println!("Task Cancelled!");
        }}
    });

    sleep(Duration::from_secs(5)).await;

    //tx.send(()) indicates "Hey, I sent a signal," without transmitting any data. The value doesn't matter if you are only using the channel as a signal.
    let _ = tx.send(());

    task.await.unwrap(); //Ensures the spawned task completes. If you don't await the task, it runs in the background.

    //------------------------------------------------------------- Channels
    //  If messages are sent faster than they are received, the channel will store them. Once the 32 messages are stored in the channel,
    // calling send(...).await will go to sleep until a message has been removed by the receiver.
    let (tx, mut rx) = mpsc::channel(32);
    let tx2 = tx.clone(); //Sending from multiple tasks is done by cloning the Sender

    tokio::spawn(async move {
        tx.send("sending from first handle").await.unwrap();
    });

    tokio::spawn(async move {
        tx2.send("sending from second handle").await.unwrap();
    });

    // /When every Sender has gone out of scope or has otherwise been dropped, it is no longer possible to send more messages into the channel.
    //At this point, the recv call on the Receiver will return None, which means that all senders are gone and the channel is closed.
    while let Some(message) = rx.recv().await {
        println!("GOT = {}", message);
    }
}

async fn async_work() -> i32 {
    let mut last = 0;
    for i in 0..5 {
        println!("Working... {i}");
        last = i;
        sleep(Duration::from_secs(1)).await;
    }
    last
}
