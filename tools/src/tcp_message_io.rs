use async_std::task::block_on;
use bincode::ErrorKind;
use log::{error, info, warn};
use message_io::network::{Endpoint, NetEvent, Transport};
use message_io::node::{self, NodeHandler};

use async_trait::async_trait;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

#[async_trait]
pub trait MessageHandler {
    ///tcp client should not impl this func
    async fn try_clone(&self) -> Self;

    ///this func just for tcp client
    async fn connect(&mut self, transport: TransportWay, addr: &str) {
        let transport = match transport {
            TransportWay::Tcp => Transport::Tcp,
            TransportWay::Udp => Transport::Udp,
        };
        let (handler, listener) = node::split::<()>();

        let (server, _) = handler.network().connect(transport, addr).unwrap();

        listener.for_each(move |event| match event.network() {
            NetEvent::Connected(endpoint, ok) => match ok {
                true => {
                    let th = TcpHandler::new(handler.clone(), server);
                    block_on(self.on_open(th));
                    info!("connect server({:?}) success!", endpoint.addr());
                }
                false => {
                    warn!("connect server({:?}) fail!", endpoint.addr());
                }
            },
            NetEvent::Message(_endpoint, data) => {
                block_on(self.on_message(data));
            }
            NetEvent::Disconnected(endpoint) => {
                block_on(self.on_close());
                info!("disconnect with server({:?})!", endpoint.addr());
            }
            _ => {}
        });
    }

    ///Triggered when there is a new client connection
    async fn on_open(&mut self, tcp_handler: TcpHandler);

    ///Disconnect triggered when client was closed
    async fn on_close(&mut self);

    ///Triggered when there is client data transfer
    ///return the res of verify,if true,that is ok,false is verify fail!
    async fn on_message(&mut self, mess: &[u8]);
}

#[derive(Clone)]
pub struct TcpHandler {
    pub node_handler: NodeHandler<()>,
    pub endpoint: Endpoint,
}

impl TcpHandler {
    pub fn new(node_handler: NodeHandler<()>, endpoint: Endpoint) -> Self {
        TcpHandler {
            node_handler: node_handler,
            endpoint: endpoint,
        }
    }

    pub fn send(&self, mess: &[u8]) {
        let endpoint = self.endpoint;
        self.node_handler.network().send(endpoint, mess);
    }
}

#[derive(Serialize, Deserialize)]
pub enum FromClientMessage {
    Ping,
}

#[derive(Serialize, Deserialize)]
pub enum FromServerMessage {
    Pong(usize), // Used for connection oriented protocols
    UnknownPong, // Used for non-connection oriented protocols
}

struct ClientInfo {
    count: usize,
}

pub enum TransportWay {
    Tcp,
    Udp,
}

pub fn run(transport: TransportWay, addr: &str, handler: impl MessageHandler) {
    let address = SocketAddr::from_str(addr);
    if let Err(e) = address {
        error!("{:?}", e);
        return;
    }

    let transport = match transport {
        TransportWay::Tcp => Transport::Tcp,
        TransportWay::Udp => Transport::Udp,
    };
    let address = address.unwrap();
    let (node_handler, listener) = node::split::<()>();

    let mut clients: HashMap<Endpoint, ClientInfo> = HashMap::new();
    let mut handler_map = HashMap::new();
    match node_handler.network().listen(transport, address) {
        Ok((_id, real_addr)) => info!("Server running at {} by {}", real_addr, transport),
        Err(e) => {
            return error!(
                "Can not listening at {} by {}!error:{:?}",
                addr, transport, e
            )
        }
    }

    listener.for_each(move |event| match event.network() {
        NetEvent::Connected(_, _) => {} // Only generated at connect() calls.
        NetEvent::Accepted(endpoint, _listener_id) => {
            // Only connection oriented protocols will generate this event
            clients.insert(endpoint, ClientInfo { count: 0 });

            let mut hd = block_on(handler.try_clone());
            //trigger the open event
            let th = TcpHandler::new(node_handler.clone(), endpoint);
            block_on(hd.on_open(th));
            handler_map.insert(endpoint, hd);
            info!("Accepted connection from: {:?}", endpoint.addr());
        }
        NetEvent::Message(endpoint, input_data) => {
            let message: Result<FromClientMessage, Box<ErrorKind>> =
                bincode::deserialize(&input_data);
            if let Ok(message) = message {
                match message {
                    FromClientMessage::Ping => {
                        let message = match clients.get_mut(&endpoint) {
                            Some(client) => {
                                // For connection oriented protocols
                                client.count += 1;
                                println!("Ping from {}, {} times", endpoint.addr(), client.count);
                                FromServerMessage::Pong(client.count)
                            }
                            None => {
                                // For non-connection oriented protocols
                                println!("Ping from {}", endpoint.addr());
                                FromServerMessage::UnknownPong
                            }
                        };
                        println!("server Received: {}", String::from_utf8_lossy(input_data));
                        let output_data = bincode::serialize(&message).unwrap();
                        node_handler.network().send(endpoint, &output_data);
                    }
                }
            }

            let hd = handler_map.get_mut(&endpoint);
            match hd {
                Some(hd) => block_on(hd.on_message(input_data)),
                None => {
                    warn!("there is no hd for endpoint!endpoint:{}", endpoint);
                }
            };
        }
        NetEvent::Disconnected(endpoint) => {
            // Only connection oriented protocols will generate this event
            clients.remove(&endpoint).unwrap();
            let hd = handler_map.remove(&endpoint);
            if let Some(mut hd) = hd {
                block_on(hd.on_close());
            }
            info!(
                "client({:?}) disconnect!so remove client peer!total clients: {}",
                endpoint.addr(),
                clients.len()
            );
        }
    });
}
