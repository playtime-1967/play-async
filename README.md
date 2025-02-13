
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
or   specify a port
```sh
cargo run --bin chat
```  
The server will start listening on **127.0.0.1:6142** by default.  

Clients/Peers can connect via `nc` (Netcat), `telnet` or a custom client:  
```sh
nc 127.0.0.1 6142
```  
or specify a port  
```sh
telnet 127.0.0.1 6142
```  

Example Output:  
![chat-terminal](https://github.com/playtime-1967/play-async/blob/master/raw/chat-terminal.jpg) 

-----------
# 2- Key-value Storage Server
A lightweight asynchronous key-value storage server built using Tokio. It supports GET, SET, and DELETE operations over a TCP connection, allowing multiple clients to interact concurrently.  


## **Features**  
- Asynchronous I/O
- Thread-safe in-memory storage
- Concurrent client support
- Basic command support: `GET`, `SET`, `DEL`  
- Custom error handling and response serialization  

### **Run the Server**  
```sh
cargo run --bin key-value-storage
```
or specify a port:
```sh
cargo run --bin key-value-storage 127.0.0.1:4162
```
The server will start listening on **127.0.0.1:4162** by default.

Connect via `nc` (Netcat) or `telnet`
```sh
nc 127.0.0.1 4162
```

### **Supported Commands**  
After connecting, you can execute the following commands:

#### ✅ **Store a Value**
```
SET key value
```
Example:
```
SET foo bar
```
Response:
```
set foo = `bar`, previous: Some("old_value")
```

#### 🔍 **Retrieve a Value**
```
GET key
```
Example:
```
GET foo
```
Response:
```
foo = bar
```

#### ❌ **Delete a Key**
```
DEL key
```
Example:
```
DEL foo
```
Response:
```
deleted foo!
```

#### ❌ **Handle Nonexistent Keys**
```
GET unknown
```
```
DEL unknown
```  
Response:
```
error: no key unknown
```

Example Output:  
![key-value-storage-terminal](https://github.com/playtime-1967/play-async/blob/master/raw/key-value-storage-terminal.jpg) 
