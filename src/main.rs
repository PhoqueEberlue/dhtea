use std::net::UdpSocket;
use clap::Parser;
mod receiver;
mod consummer;

use std::sync::mpsc;
use std::sync::Arc;
use std::thread;


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

    // Local socket of the node
    let socket = Arc::new(UdpSocket::bind(local_address)?);

    // Cloning to pass the Atomic Reference Counted to the thread
    let socket_ref = socket.clone();

    // Creating a Multi Producer Single Consummer channel
    let (tx, rx) = mpsc::channel();

    // Launching the thread that listens to the others nodes
    let handle_listener = thread::spawn(move || {
        receiver::listen_request(socket_ref, tx).unwrap();
    });

    // Create remote address if both ip and port were provided, otherwise set to None
    let remote_address = match args.remote_ip.zip(args.remote_port) {
        Some((mut ip, port)) => {
            ip.push_str(&format!(":{port}"));
            Some(ip)
        },
        None => None,
    };
   
    // Cloning to pass the Atomic Reference Counted to the thread
    let socket_ref = socket.clone();

    // Launching the thread that consumme and treat messages
    let handle_consummer = thread::spawn(move || {
        consummer::main_consummer(socket_ref, remote_address, rx).unwrap();
    });

    handle_listener.join().unwrap();
    handle_consummer.join().unwrap();
    Ok(())
}


