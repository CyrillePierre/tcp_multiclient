use tcp_multiclient::*;
use tokio::runtime::Runtime;

fn main() {
    config_logger();

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
        let handles = listeners.start_accept();

        // wait the server tasks
        futures::future::join_all(handles).await;
    });
}

fn help(arg0: &str) -> ! {
    println!("Syntax: {} <port> [<port>...]", arg0);
    std::process::exit(1);
}

fn config_logger() {
    env_logger::builder()
        .format_timestamp(None)
        .filter_level(log::LevelFilter::Info)
        .filter_module("mio::poll", log::LevelFilter::Warn)
        .format_module_path(false)
        .parse_default_env()
        .init();
}
