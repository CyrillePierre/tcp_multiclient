use futures::stream::{self, StreamExt};
use log::{error, info, trace, warn};
use std::{collections::HashMap, net::SocketAddr, sync::Arc};
use tokio::{
    net::{tcp, TcpListener, TcpStream},
    prelude::*,
    sync::{Mutex, RwLock},
    task::JoinHandle,
};

pub struct Client {
    addr: SocketAddr,
    r_stream: Mutex<tcp::OwnedReadHalf>,
    w_stream: Mutex<tcp::OwnedWriteHalf>,
}

type ClientMap = HashMap<SocketAddr, Arc<Client>>;
type Clients = Arc<RwLock<ClientMap>>;
type Listeners = Vec<Arc<TcpListener>>;

pub struct NetMgr {
    listeners: Listeners,
    clients: Clients,
}

impl Client {
    pub fn new(addr: SocketAddr, sock: TcpStream) -> Client {
        let (rs, ws) = sock.into_split();
        Client {
            addr,
            r_stream: Mutex::new(rs),
            w_stream: Mutex::new(ws),
        }
    }

    pub async fn process(&self, clients: &Clients) {
        loop {
            let mut buf = [0u8; 4096];
            let size = self.r_stream.lock().await.read(&mut buf).await;
            if let Err(e) = &size {
                warn!("client {}: failed to read: {}", &self.addr, &e);
            }
            let size = size.unwrap();

            trace!("{}: read ({}) {}", &self.addr, size, format_data(&buf[..size]));

            if size == 0 {
                break;
            }

            // write the buffer to all other clients
            for (a, c) in clients.read().await.iter() {
                if *a != self.addr {
                    c.w_stream
                        .lock()
                        .await
                        .write_all(&buf[..size])
                        .await
                        .unwrap();
                }
            }
        }
    }
}

impl NetMgr {
    pub async fn generate_listeners<T>(args: T, ip: &str) -> NetMgr
    where
        T: Iterator,
        T::Item: std::fmt::Display,
    {
        // async bloc used to bind the TcpListener to each port
        let tcp_bind = |port| async move {
            let addr = format!("{}:{}", ip, port);
            match TcpListener::bind(&addr).await {
                Ok(tl) => {
                    info!("Binding {}", tl.local_addr().unwrap());
                    Some(Arc::new(tl))
                }
                Err(e) => {
                    warn!("Failed to bind {}: {}", &addr, &e);
                    None
                }
            }
        };

        // iterates on all ports (args) and bind a socket to each port
        let listeners: Listeners = stream::iter(args).filter_map(tcp_bind).collect().await;
        if listeners.is_empty() {
            error!("No IPv4 or IPv6 bind available.");
            std::process::exit(2);
        }

        NetMgr {
            listeners,
            clients: Arc::new(RwLock::new(ClientMap::new())),
        }
    }

    pub fn start_accept(&self) -> Vec<JoinHandle<()>> {
        let mut handles = vec![];
        for listener in self.listeners.iter().cloned() {
            let clients = Arc::clone(&self.clients);
            handles.push(tokio::spawn(async move {
                loop {
                    // accept a new TCP client
                    let (sock, addr) = listener.accept().await.unwrap();
                    let client = Client::new(addr, sock);
                    clients.write().await.insert(addr, Arc::new(client));

                    info!("new client: {}", &addr);

                    let clients = Arc::clone(&clients);
                    tokio::spawn(async move {
                        // handle the client (read loop)
                        let client = Arc::clone(clients.read().await.get(&addr).unwrap());
                        client.process(&clients).await;

                        // when process is finished, the client is removed and closed
                        let mut clients = clients.write().await;
                        clients.remove(&addr);

                        info!("disconnect client: {}", &addr);
                    });
                }
            }));
        }
        handles
    }
}

fn format_data(data: &[u8]) -> String {
    const MAX_WIDTH: usize = 40;

    let mut overflow = false;
    let mut res = String::new();
    res.push('"');
    for c in data {
        if res.len() >= MAX_WIDTH { overflow = true; break; }
        match *c {
            b'\n' => res.push_str(r"\n"),
            b'\t' => res.push_str(r"\t"),
            b'\r' => res.push_str(r"\r"),
            b'\\' => res.push_str(r"\\"),
            c if c >= b' ' && c.is_ascii() => res.push(c as char),
            c => res.push_str(&format!("\\x{:02x}", c)),
        }
    }

    res.push('"');
    if overflow { res.push_str("..."); }
    res
}
