use tcp_multiclient::*;
use tokio::{io::stdin, prelude::*, runtime::Runtime};

fn main() {
    let mut args = std::env::args().peekable();
    let prog_name = args
        .next()
        .unwrap_or_else(|| help("./<name_of_executable>"));
    let ip = "[::]"; // IPv6 catchall

    // check if there is an argument
    args.peek().unwrap_or_else(|| help(&prog_name));

    let rt = Runtime::new().unwrap();
    rt.block_on(async move {
        // generate TcpListeners from the ports
        let listeners = NetMgr::generate_listeners(args, ip).await;
        listeners.start_accept();
        stdin().read_u8().await.ok();
    });
}

fn help(arg0: &str) -> ! {
    eprintln!("Syntax: {} <port> [<port2> ...]", arg0);
    std::process::exit(1);
}
