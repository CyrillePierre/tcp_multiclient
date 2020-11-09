use tokio::{runtime::Runtime};
use tcp_multiclient::*;

fn main() {
    let mut args = std::env::args().peekable();
    let prog_name = args
        .next()
        .unwrap_or_else(|| help("./<name_of_executable>"));
    let ip = "[::]"; // IPv6 catchall

    // check if there is an argument
    args.peek().unwrap_or_else(|| help(&prog_name));

    let rt  = Runtime::new().unwrap();
    rt.block_on(async move {
        // generate TcpListeners from the ports
        let servers = generate_listeners(args, ip);
        println!("{:?}", servers.await);
    });
}

fn help(arg0: &str) -> ! {
    eprintln!("Syntax: {} <port> [<port2> ...]", arg0);
    std::process::exit(1);
}
