
# Async programming with Tokio
Covering cross-task communication, async channels, locks, TCP/UDP sockets, and task cancellation; implemented a chat server with user notifications and message broadcasting, and a HashMap server allowing clients to perform CRUD operations on data.

-------------
# 1- Chat App  

This is an asynchronous chat server built using [Tokio](https://tokio.rs/), leveraging `TcpListener`, `TcpStream`, and `mpsc` channels for efficient message handling. The server supports multiple clients and broadcasts messages while maintaining a simple peer-to-peer communication structure.  

## Features  
- Asynchronous message handling with Tokio  
- Multi-client support with non-blocking I/O  
- Broadcast system for real-time chat  

## Steps to run  
Run the chat server with:  
```sh
cargo run --bin chat 127.0.0.1:6142
```  
or  
```sh
cargo run --bin chat
```  
Clients/Peers can connect via `nc` (Netcat), `telnet` or a custom client:  
```sh
nc 127.0.0.1 6142
```  
or  
```sh
telnet 127.0.0.1 6142
```  

Example Output:  
![chat-terminal](https://github.com/playtime-1967/play-async/blob/master/raw/chat-terminal.jpg) 
