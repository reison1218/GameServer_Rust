use mio::*;
use mio::tcp::{TcpListener, TcpStream};
use std::io::Read;
use std::sync::{Arc, RwLock};
use std::collections::HashMap;

const SERVER: Token = Token(0);
const CLIENT: Token = Token(1);
const STRESM: Token = Token(2);
pub fn test_mio(){

    let addr = "127.0.0.1:16801".parse().unwrap();

// Setup the server socket
    let server = TcpListener::bind(&addr).unwrap();

// Create an poll instance
    let poll = Poll::new().unwrap();

    let poll_stream = Arc::new(Poll::new().unwrap());

// Start listening for incoming connections
    poll.register(&server, SERVER, Ready::readable(),
                  PollOpt::edge()).unwrap();

// Setup the client socket
    let sock = TcpStream::connect(&addr).unwrap();

// Register the socket
    poll.register(&sock, CLIENT, Ready::readable(),
                  PollOpt::edge()).unwrap();

// Create storage for events
    let mut events = Events::with_capacity(1024);


    let mut map = Arc::new(RwLock::new(HashMap::new()));

    let mut map_copy = map.clone();
    let poll_stream_clone = poll_stream.clone();
    let (send,rec) = std::sync::mpsc::sync_channel(1);
    let m = move ||{
        loop {
            poll.poll(&mut events, None).unwrap();

            for event in events.iter() {
                match event.token() {
                    SERVER => {
                        // Accept and drop the socket immediately, this will close
                        // the socket and notify the client of the EOF.
                        let con=   server.accept().unwrap();
                        poll_stream_clone.register(&con.0, STRESM, Ready::readable(),
                                      PollOpt::edge()).unwrap();
                        map_copy.write().unwrap().insert(con.1.to_string(),con.0);
                        send.send(con.1.to_string());
                    }
                    CLIENT => {
                        // The server just shuts down the socket, let's just exit
                        // from our event loop.
                        return;
                    }
                    STRESM =>{

                    },
                    _ => unreachable!(),
                }
            }
        }
    };
    std::thread::spawn(m);
    let mut events_stream = Events::with_capacity(1024);
    loop{
        poll_stream.poll(&mut events_stream, None).unwrap();

        for event in events_stream.iter() {
            match event.token(){
                STRESM=>{
                    let key = rec.recv().unwrap();
                    let mut write = map.write().unwrap();
                    let res = write.get_mut(key.as_str()).unwrap();
                    let mut bytes:[u8;1024] = [0;1024];
                    res.read(&mut bytes);
                },
                _=>unreachable!(),
            };

        }
    }

}