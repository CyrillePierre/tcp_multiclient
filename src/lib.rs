use tokio::{net::TcpListener};
use futures::stream::{self, StreamExt};

pub async fn generate_listeners<T>(args: T, ip: &str) -> Vec<TcpListener>
where
    T: Iterator,
    T::Item: std::fmt::Display,
{
    let tcp_bind = |port| async move {
        let addr = format!("{}:{}", ip, port);
        match TcpListener::bind(&addr).await {
            Ok(tl) => {
                println!("Binding {}", tl.local_addr().unwrap());
                Some(tl)
            }
            Err(e) => {
                eprintln!("Failed to bind {}: {}", addr, e);
                None
            }
        }
    };

    let servers: Vec<TcpListener> = stream::iter(args).filter_map(tcp_bind).collect().await;
    if servers.is_empty() {
        eprintln!("No IPv4 or IPv6 bind available.");
        std::process::exit(2);
    }

    servers
}
