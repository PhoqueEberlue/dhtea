mod dht;

use clap::Parser;
use dht::Node;

// Declaring arguments
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// binding port of the local machine
    #[arg(long)]
    bind_port: String,

    /// IP Address of remote node
    #[arg(long)]
    remote_ip: Option<String>,

    /// Port of remote node
    #[arg(long)]
    remote_port: Option<String>,
}

fn main() -> std::io::Result<()> {
    // Parsing arguments
    let args = Args::parse();

    // Building local address
    let addr: dht::Addr = dht::Addr {
        ip: String::from("127.0.0.1"),
        port: args.bind_port.parse().unwrap(),
    };

    // Create remote address if both ip and port were provided, otherwise set to None
    let remote_address = match args.remote_ip.zip(args.remote_port) {
        Some((ip, port)) => Some(dht::Addr {
            ip,
            port: port.parse().unwrap(),
        }),
        None => None,
    };

    let mut node = Node::new(addr);
    node.run(remote_address)?;

    Ok(())
}
