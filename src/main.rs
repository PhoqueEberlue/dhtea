mod dht;

use clap::Parser;
use dht::Node;

// Declaring arguments
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// binding ip of the local machine
    #[arg(long)]
    bind_ip: String,

    /// binding port of the local machine
    #[arg(long)]
    bind_port: String,

    /// IP Address of remote node
    #[arg(long)]
    remote_ip: Option<String>,

    /// Port of the remote node
    #[arg(long)]
    remote_port: Option<String>,
}

fn main() -> std::io::Result<()> {
    // Parsing arguments
    let args = Args::parse();

    // Building local address
    let mut local_address = args.bind_ip.clone();
    local_address.push_str(&format!(":{}", args.bind_port));

    // Create remote address if both ip and port were provided, otherwise set to None
    let remote_address = match args.remote_ip.zip(args.remote_port) {
        Some((mut ip, port)) => {
            ip.push_str(&format!(":{port}"));
            Some(ip)
        }
        None => None,
    };

    let node = Node::new(local_address);
    node.run(remote_address)?;

    Ok(())
}
