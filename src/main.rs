use std::net::{TcpListener, };

fn main() {
    let mut args = std::env::args().peekable();
    let prog_name = args
        .next()
        .unwrap_or_else(|| help("./<name_of_executable>"));
    let ip = "[::]"; // IPv6 catchall

    // check if there is an argument
    args.peek().unwrap_or_else(|| help(&prog_name));

    // generate TcpListeners from the ports
    let servers = generate_listeners(args, ip);
    println!("{:?}", servers);
}

fn help(arg0: &str) -> ! {
    eprintln!("Syntax: {} <port> [<port2> ...]", arg0);
    std::process::exit(1);
}

fn generate_listeners<T>(args: T, ip: &str) -> Vec<TcpListener>
where
    T: Iterator,
    T::Item: std::fmt::Display,
{
    let servers: Vec<TcpListener> = args
        .filter_map(|port| {
            let addr = format!("{}:{}", ip, port);
            match TcpListener::bind(&addr) {
                Ok(tl) => {
                    println!("Binding {}", tl.local_addr().unwrap());
                    Some(tl)
                }
                Err(e) => {
                    eprintln!("Failed to bind {}: {}", addr, e);
                    None
                }
            }
        })
        .collect();
    if servers.is_empty() {
        eprintln!("No IPv4 or IPv6 bind available.");
        std::process::exit(2);
    }

    servers
}
