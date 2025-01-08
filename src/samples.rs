use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::spawn;
use tokio::sync::mpsc;
use tokio::sync::oneshot;
use tokio::time::{sleep, Duration};
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

    //------------------------------------------- I/O
    //Read some bytes
    let mut f = File::open("src/foo.txt").await.unwrap();
    let mut buffer = [0; 10];
    //When read() returns Ok(0), this signifies that the stream is closed. Any further calls to read() will complete immediately with Ok(0)
    let size = f.read(&mut buffer).await.unwrap(); // returning the number of bytes read
    println!("The bytes:{size} {:?}", &buffer[..size]);

    //read_to_end
    let mut buffer = Vec::new();

    f.read_to_end(&mut buffer).await.unwrap();
    println!("The bytes {:?}", &buffer[..]);

    //Write
    let mut f = File::create("src/foo2.txt").await.unwrap();

    // Writes some prefix of the byte string, but not necessarily all of it.
    let n = f.write(b"bye bytes.").await.unwrap();
    println!("Wrote the first {} bytes of 'bye bytes'.", n);

    //writes the entire buffer into the writer
    f.write_all(b"bye bytes.").await.unwrap();

    //tokio::io::copy asynchronously copies the entire contents of a reader into a writer.
    let mut reader: &[u8] = b"hello";
    let mut file = File::create("src/foo.txt").await.unwrap();

    tokio::io::copy(&mut reader, &mut file).await.unwrap();
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
