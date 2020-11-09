use futures::stream::{self, StreamExt};
use tokio::{
    prelude::*,
    sync::Mutex, 
    net::{TcpListener, TcpStream},
};
use std::{collections::HashMap, net::SocketAddr, sync::Arc};

type Client = Arc<TcpStream>;
type ClientMap = HashMap<SocketAddr, Client>;
type Clients = Arc<Mutex<ClientMap>>;
type Listeners = Vec<Arc<TcpListener>>;

pub struct NetMgr {
    listeners: Listeners,
    clients: Clients,
}

impl NetMgr {
    pub async fn generate_listeners<T>(args: T, ip: &str) -> NetMgr
    where
        T: Iterator,
        T::Item: std::fmt::Display,
    {
        let tcp_bind = |port| async move {
            let addr = format!("{}:{}", ip, port);
            match TcpListener::bind(&addr).await {
                Ok(tl) => {
                    println!("Binding {}", tl.local_addr().unwrap());
                    Some(Arc::new(tl))
                }
                Err(e) => {
                    eprintln!("Failed to bind {}: {}", addr, e);
                    None
                }
            }
        };

        let listeners: Listeners = stream::iter(args)
            .filter_map(tcp_bind).collect().await;
        if listeners.is_empty() {
            eprintln!("No IPv4 or IPv6 bind available.");
            std::process::exit(2);
        }
        
        NetMgr{listeners, clients: Arc::new(Mutex::new(ClientMap::new()))}
    }

    pub fn start_accept(&self) {
        for listener in self.listeners.iter().cloned() {
            let clients = Arc::clone(&self.clients);
            tokio::spawn(async move {
                loop {
                    let (sock, addr) = listener.accept().await.unwrap();
                    clients.lock().await.insert(addr, Arc::new(sock));

                    let clients = Arc::clone(&clients);
                    tokio::spawn(async move {
                        let client = Arc::clone(clients.lock().await.get(&addr).unwrap());
                        process_client(Arc::clone(&clients), client).await;
                        let mut clients = clients.lock().await;
                        clients.remove(&addr);
                    });
                }
            });
        }
    }

}

pub async fn process_client(clients: Clients, mut client: Client) {
    let mut buf = [0; 4096];
    let client = Arc::get_mut(&mut client).unwrap();
    let addr = &client.local_addr().unwrap();

    let size = client.read(&mut buf).await;
    if let Err(e) = &size { eprintln!("failed to read: {}", e); }
    let size = size.unwrap();

    // write the buffer to all other clients
    for (a, c) in clients.lock().await.iter() {
        let mut c = Arc::clone(c);
        let c = Arc::get_mut(&mut c).unwrap();
        if a != addr { c.write_all(&buf[..size]).await.unwrap(); }
    }
}
