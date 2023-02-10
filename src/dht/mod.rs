mod req_handler;
mod req_listener;

use std::net::UdpSocket;
use std::sync::mpsc;
use std::sync::Arc;
use std::thread;

pub struct Node {
    local_address: String,
    remote_address: Option<String>,
    socket: Arc<UdpSocket>,
}

impl Node {
    pub fn new(local_address: String, remote_address: Option<String>) -> Self {
        // Local socket of the node
        let socket = Arc::new(match UdpSocket::bind(&local_address) {
            Ok(s) => s,
            Err(e) => panic!("{}", e),
        });

        Node {
            local_address,
            remote_address,
            socket,
        }
    }

    pub fn run(&self) -> std::io::Result<()> {
        // Cloning to pass the Atomic Reference Counted to the thread
        let socket_ref = self.socket.clone();

        // Creating a Multi Producer Single Consummer channel
        let (tx, rx) = mpsc::channel();

        // Launching the thread that listens to the others nodes
        let handle_listener = thread::spawn(move || {
            req_listener::run(socket_ref, tx).unwrap();
        });

        // Cloning to pass the Atomic Reference Counted to the thread
        let socket_ref = self.socket.clone();

        let remote_address_copy = self.remote_address.clone();

        // Launching the thread that consumme and treat messages
        let handle_consummer = thread::spawn(move || {
            req_handler::run(socket_ref, remote_address_copy, rx).unwrap();
        });

        handle_listener.join().unwrap();
        handle_consummer.join().unwrap();

        Ok(())
    }
}
