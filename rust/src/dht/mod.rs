mod req_listener;

use std::net::UdpSocket;
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
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
    address: String //ok but more perf if port and addr dissociated
}

/// represents a node (ie a machine) in the dht
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

    /// runs a node indefenitely
    pub fn run(&self, remote_address: Option<String>) -> std::io::Result<()> {
        // Cloning to pass the Atomic Reference Counted to the thread
        let socket_ref = self.socket.clone();

        // Creating a Multi Producer Single Consummer channel
        let (tx, rx) = mpsc::channel();

        // Launching the thread that listens to the others nodes
        let handle_listener = thread::spawn(move || {
            req_listener::run(socket_ref, tx).unwrap();
        });

        self.run_req_handler(remote_address, rx).unwrap();

        handle_listener.join().unwrap();

        Ok(())
    }

    // Main function of the request handler
    fn run_req_handler(
        &self,
        remote_address: Option<String>,
        channel_consummer: Receiver<String>,
    ) -> std::io::Result<()> {
        // If remote_address is not None, unwraps into address
        if let Some(address) = remote_address {
            let msg = "Whats up";
            self.socket.connect(address.clone())?;
            self.socket.send(msg.as_bytes())?;
            println!("Sending {:?} to {:?}", msg, address);
        }

        loop {
            Node::handle_request(&channel_consummer);
        }
    }

    fn handle_request(channel_consummer: &Receiver<String>) {
        let rcv = channel_consummer.recv().unwrap();//i like naming var ;)
        let mut rcv = rcv.split(";");//split at end of source; there is no ; in an address, we good
        let src = rcv.next().unwrap().replacen("SRC:", "", 1);//shadowing go brrrrrr
        let msg = rcv.next().unwrap().replacen("MSG:", "", 1);//replace only the first occurence

        println!("recieved {:?} from {:?}", msg, src);
    }
}
