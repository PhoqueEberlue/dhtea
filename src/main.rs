use std::net::UdpSocket;
use clap::Parser;


#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// IP Address of the node to connect to
    #[arg(short, long)]
    ip: String,

    /// Port of the node to connect to
    #[arg(short, long)]
    port: String,
}

fn main() -> std::io::Result<()> {
    let args = Args::parse();
    println!("IP: {}", args.ip);
    println!("Port: {}", args.port);
    Ok(())
}
