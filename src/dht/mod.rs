mod req_handler;
mod req_listener;

use std::net::UdpSocket;
use std::sync::mpsc;
use std::sync::Arc;
use std::thread;
use uuid::Uuid;

/// Struct representing the local node
pub struct Node {
    address: String,
    socket: Arc<UdpSocket>,
    id: Uuid,
    left_neighbour: Option<Neighbour>,
    right_neighbour: Option<Neighbour>,
}

/// Struct storing neighbour node's informations
pub struct Neighbour {
    address: String,
    id: Uuid,
}

impl Node {
    pub fn new(local_address: String) -> Self {
        // Local socket of the node
        let socket = Arc::new(match UdpSocket::bind(&local_address) {
            Ok(s) => s,
            Err(e) => panic!("{}", e),
        });

        Node {
            address: local_address,
            socket,
            id: Uuid::new_v4(),
            left_neighbour: None,
            right_neighbour: None,
        }
    }

    pub fn run(&self, remote_address: Option<String>) -> std::io::Result<()> {
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

        // Launching the thread that consumme and treat messages
        let handle_consummer = thread::spawn(move || {
            req_handler::run(socket_ref, remote_address, rx).unwrap();
        });

        handle_listener.join().unwrap();
        handle_consummer.join().unwrap();

        Ok(())
    }
}
