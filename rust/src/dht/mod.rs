mod req_listener;

use std::net::UdpSocket;
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::sync::Arc;
use std::thread;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

#[derive(Hash, Clone)]
pub struct Addr {
    pub ip:String,
    pub port: u32
}

impl Addr {
    pub fn to_string(&self) -> String {
        return format!("{}:{}", self.ip, self.port);
    }
}

pub struct Node {
    hash: u64,
    address: Addr,
    socket: Arc<UdpSocket>,
    left_neighbour: Option<Neighbour>,
    right_neighbour: Option<Neighbour>,
}

/// Struct storing neighbour node's informations
#[derive(Clone)]
pub struct Neighbour {
    hash: u64, // not option, HAS TO BE there
    address: Addr //ok but more perf if port and addr dissociated
                  //who cares its only a few cpu cycles anyway
}

/// represents a node (ie a machine) in the dht
impl Node {
    pub fn new(addr: Addr) -> Self {
        // Local socket of the node
        let mut dest: String = addr.clone().ip;
        dest.push_str(":");
        dest.push_str(addr.port.to_string().as_str());

        let socket = Arc::new(match UdpSocket::bind(dest.clone()) {
            Ok(s) => s,
            Err(e) => panic!("ip {} error {}", dest, e),
        });

        Node {
            hash: calculate_hash(&addr),
            address: addr,
            socket,
            left_neighbour: None,
            right_neighbour: None,
        }
    }

    /// runs a node indefenitely
    pub fn run(&mut self, remote_address: Option<Addr>) -> std::io::Result<()> {
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
        &mut self,
        remote_address: Option<Addr>,
        channel_consummer: Receiver<String>,
    ) -> std::io::Result<()> {
        // If remote_address is not None, unwraps into address
        if let Some(address) = remote_address {
            let msg = "INIT CONNECT";
            self.socket.send_to(msg.as_bytes(), address.to_string())?;
            println!("Sending {:?} to {:?}:{:?}", msg, address.ip, address.port);
        }

        loop {
            self.handle_request(&channel_consummer);
        }
    }
    /// DO NOT USE THAT we dont want to block to single addr
    fn connect(&self, addr: Addr) {
        let mut dest: String = addr.ip;
        dest.push_str(":");
        dest.push_str(addr.port.to_string().as_str());

        self.socket.connect(dest).unwrap();
    }

    fn handle_request(&mut self, channel_consummer: &Receiver<String>) {
        let rcv = channel_consummer.recv().unwrap();//i like naming var ;)
        let mut rcv = rcv.split(";");//split at end of source; there is no ; in an address, we good
        let src = rcv.next().unwrap().replacen("SRC:", "", 1);//shadowing go brrrrrr
        let msg = rcv.next().unwrap().replacen("MSG:", "", 1);//replace only the first occurence

        let mut src_splt = src.split(":");
        let ip = src_splt.next().unwrap().replacen(":", "", 1);
        let port = src_splt.next().unwrap().parse().unwrap();

        let addr: Addr = Addr{ ip, port};

        match msg.as_str() {
            "JOIN" => {
                print!("[JOIN REQUEST]");
            },
            "INIT CONNECT" => {
                print!("[INIT CONNECT]");

                self.join_other(addr);
            },
            _ => panic!("not handled yet {:?}", msg)
        }

    }

    /// first join request, everything else is called from here
    fn join_other(&mut self, entry: Addr) {
        let hash = calculate_hash(&entry);
        if hash > self.hash {
            self.join_other_right(entry, hash);
        }else {
            self.insert_left(hash, entry);//TODO join left
        }
    }

    fn insert_left(&mut self, hash: u64, addr: Addr) {
        match &mut self.left_neighbour {
            Some(neighbour) => {
                neighbour.hash = hash;
                neighbour.address = addr.clone();

                self.send_insert_left(addr.clone()); //send req to other left
            },
            None => {
                self.left_neighbour = Some(Neighbour{ hash, address: addr.clone() });
            }
        }

        let src = self.address.clone();
        println!("[INSERT] {} is left of {}",
                 src.to_string(), addr.to_string());
    }

    fn insert_right(&mut self, hash: u64, addr: Addr) {
        match &mut self.right_neighbour {
            Some(neighbour) => {
                neighbour.hash = hash;
                neighbour.address = addr.clone();

                self.send_insert_right(addr.clone()); //send req to other right
            },
            None => {
                self.right_neighbour = Some(Neighbour{ hash, address: addr.clone() });
            }
        }

        let src = self.left_neighbour.clone().unwrap().address;
        println!("[INSERT] {} is right of {}",
                 src.to_string(), addr.to_string());
    }

    /// check and insert if node should be inserted right
    /// if not, sends it to right node to check for the same thing
    fn join_other_right(&self, to_join: Addr, hash: u64) {

    }

    fn join_other_left(&self, to_join: Addr, hash: u64) {

    }

    fn send_insert_right(&self, addr: Addr) -> Option<()> {
        let dest = self.right_neighbour.clone()?.address;
        let msg = format!("INS:RGT:{}:{}", addr.ip, addr.port);
        self.socket.send_to(msg.as_bytes(), dest.to_string()).unwrap();
        println!("Sending {:?} to {:?}:{:?}", msg, addr.ip, addr.port);
        Some(())
    }

    /// send a request to left node to insert it to its right
    /// this way is safer (nodes have to be reviewed by peer)
    /// other way would be to return address to other and tell 
    /// the requesing node to go insert itself
    fn send_insert_left(&self, addr: Addr) -> Option<()> {
        let dest = self.right_neighbour.clone()?.address;
        let msg = format!("INS:RGT:{}:{}", addr.ip, addr.port);
        self.socket.send_to(msg.as_bytes(), dest.to_string()).unwrap();
        println!("Sending {:?} to {:?}:{:?}", msg, addr.ip, addr.port);
        Some(())
    }

    fn join_left(&self) {

    }
}

fn calculate_hash<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}
