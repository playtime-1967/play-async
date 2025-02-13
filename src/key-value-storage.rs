use futures::SinkExt;
use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;
use tokio_stream::StreamExt;
use tokio_util::codec::{Framed, LinesCodec};

//to mutate the internal map we're going to use a `Mutex`.
struct Database {
    map: Mutex<HashMap<String, String>>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:4162".to_string());

    let listener = TcpListener::bind(&addr).await?;
    println!("Listening on: {addr}");

    //shared state amongst all clients.
    //ensure each client will have a reference to the storage.
    let mut initial_db = HashMap::new();
    initial_db.insert("foo".to_string(), "bar".to_string());
    let db = Arc::new(Database {
        map: Mutex::new(initial_db),
    });

    loop {
        match listener.accept().await {
            Ok((socket, _)) => {
                //creating a new reference of the storage for the connected client.
                let db = db.clone();

                tokio::spawn(async move {
                    //convert the stream of bytes into a stream of lines.
                    let mut lines = Framed::new(socket, LinesCodec::new());

                    while let Some(result) = lines.next().await {
                        match result {
                            Ok(line) => {
                                let response = handle_request(&line, &db);

                                let response = response.serialize();
                                //return back the response to the client.
                                if let Err(e) = lines.send(response.as_str()).await {
                                    println!("error on sending response; error = {e:?}");
                                }
                            }
                            Err(e) => {
                                println!("error on decoding from socket; error = {e:?}");
                            }
                        }
                    }

                    //the connection will be closed at this point as `lines.next()` has returned `None`.
                });
            }
            Err(e) => println!("error accepting socket; error = {e:?}"),
        }
    }
}

fn handle_request(line: &str, db: &Arc<Database>) -> Response {
    let request = match Request::parse(line) {
        Ok(req) => req,
        Err(e) => return Response::Error { msg: e },
    };

    let mut db = db.map.lock().unwrap();
    match request {
        Request::Get { key } => match db.get(&key) {
            Some(value) => Response::Value {
                key,
                value: value.clone(),
            },
            None => Response::Error {
                msg: format!("no key {key}"),
            },
        },
        Request::Set { key, value } => {
            let previous = db.insert(key.clone(), value.clone());
            Response::Set {
                key,
                value,
                previous,
            }
        }
        Request::Del { key } => match db.get(&key) {
            Some(_) => {
                db.remove(&key).expect("msg");
                Response::Del { key }
            }
            None => Response::Error {
                msg: format!("no key {key}"),
            },
        },
    }
}


enum Request {
    Get { key: String },
    Set { key: String, value: String },
    Del { key: String },
}

impl Request {
    fn parse(input: &str) -> Result<Request, String> {
        let mut parts = input.splitn(3, ' ');
        match parts.next() {
            Some("GET") => {
                let key = parts.next().ok_or("GET must be followed by a key")?;
                if parts.next().is_some() {
                    return Err("GET's key must not be followed by anything".into());
                }
                Ok(Request::Get {
                    key: key.to_string(),
                })
            }
            Some("SET") => {
                let key = match parts.next() {
                    Some(key) => key,
                    None => return Err("SET must be followed by a key".into()),
                };
                let value = match parts.next() {
                    Some(value) => value,
                    None => return Err("SET needs a value".into()),
                };
                Ok(Request::Set {
                    key: key.to_string(),
                    value: value.to_string(),
                })
            }
            Some("DEL") => {
                let key = parts.next().ok_or("GET must be followed by a key")?;
                if parts.next().is_some() {
                    return Err("DEl's key must not be followed by anything".into());
                }
                Ok(Request::Del {
                    key: key.to_string(),
                })
            }
            Some(cmd) => Err(format!("unknown command: {cmd}")),
            None => Err("empty input".into()),
        }
    }
}

enum Response {
    Value {
        key: String,
        value: String,
    },
    Set {
        key: String,
        value: String,
        previous: Option<String>,
    },
    Del {
        key: String,
    },
    Error {
        msg: String,
    },
}

impl Response {
    fn serialize(&self) -> String {
        match *self {
            Response::Value { ref key, ref value } => format!("{key} = {value}"),
            Response::Set {
                ref key,
                ref value,
                ref previous,
            } => format!("set {key} = `{value}`, previous: {previous:?}"),
            Response::Del { ref key } => format!("deleted {key}!"),
            Response::Error { ref msg } => format!("error: {msg}"),
        }
    }
}
