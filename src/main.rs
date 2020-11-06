use std::net::{TcpListener, TcpStream};

fn main() {
    let mut args = std::env::args();
    let prog_name = args.next().unwrap_or_else(|| help("./<name_of_executable>"));
    let port = args.next().unwrap_or_else(|| help(&prog_name));

    // generate TcpListeners from the port
    let servers: Vec<TcpListener> = ["[::1]", "0.0.0.0"].iter()
        .filter_map(|ip| 
            match TcpListener::bind(format!("{}:{}", ip, port)) {
                Ok(tl) => { println!("Binding {}", tl.local_addr().unwrap()); Some(tl) },
                Err(e) => { eprintln!("Failed to bind {}: {}", ip, e); None }
            })
        .collect();
    if servers.is_empty() {
        eprintln!("No IPv4 or IPv6 bind available for TCP port '{}'.", port);
        std::process::exit(2);
    }
    println!("{:?}", servers);
}

fn help(arg0: &str) -> ! {
    eprintln!("Syntax: {} <port>", arg0);
    std::process::exit(1);
}
