use std::net::{TcpListener, TcpStream};

fn main() {
    let mut args = std::env::args().peekable();
    let prog_name = args.next().unwrap_or_else(|| help("./<name_of_executable>"));
    let ip = "[::]";

    // check if there is an argument
    args.peek().unwrap_or_else(|| help(&prog_name));

    // generate TcpListeners from the port
    //let servers: Vec<TcpListener> = ["[::1]", "0.0.0.0", "127.0.0.1"].iter()
    let servers: Vec<TcpListener> = args
        .filter_map(|port| {
            let addr = format!("{}:{}", ip, port);
            match TcpListener::bind(&addr) {
                Ok(tl) => { println!("Binding {}", tl.local_addr().unwrap()); Some(tl) },
                Err(e) => { eprintln!("Failed to bind {}: {}", addr, e); None }
            }
        })
        .collect();
    if servers.is_empty() {
        eprintln!("No IPv4 or IPv6 bind available.");
        std::process::exit(2);
    }
    println!("{:?}", servers);
}

fn help(arg0: &str) -> ! {
    eprintln!("Syntax: {} <port> [<port2> ...]", arg0);
    std::process::exit(1);
}
