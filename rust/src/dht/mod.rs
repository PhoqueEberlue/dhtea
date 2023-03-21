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
        let hash = calculate_hash(&addr);

        println!("[HANDLE REQUEST] self {} req {} msg {}", self.hash, hash, msg);

        let mut msg = msg.split(":");

        match msg.next().unwrap() {
            "JOIN" => { //TODO join should be insert, and insert reqinsert ?
                let direction = msg.next().unwrap();

                let ip = msg.next().unwrap();
                let port = msg.next().unwrap();

                let addr = Addr{ ip: String::from(ip), port: port.parse().unwrap() };
                let hash = calculate_hash(&addr);

                match direction {
                    "LFT" => self.insert_left(addr, hash, true),
                    "RGT" => self.insert_right(addr, hash, true),
                    _ => panic!("join fail")
                }
            },
            "INIT CONNECT" => {
                print!("[INIT CONNECT]");
                self.join_other(addr);
            },
            "REQINS" => {
                let direction = msg.next().unwrap();
                let ip = msg.next().unwrap();
                let port = msg.next().unwrap();
                let addr = Addr{ip: String::from(ip), port: port.parse().unwrap()};
                
                match direction {
                    "RGT" => { self.join_other_right(addr.clone(), hash) },
                    "LFT" => { self.join_other_left(addr.clone(), hash) },
                    _ => panic!("didnt understand direction in join. expected rgt or lft got {:?}", direction)
                }
            },
            "INS" => {
                let direction = msg.next().unwrap();
                let ip = msg.next().unwrap();
                let port = msg.next().unwrap();
                let addr = Addr{ip: String::from(ip), port: port.parse().unwrap()};
                
                match direction {
                    "RGT" => { self.insert_right(addr.clone(), hash, true) },
                    "LFT" => { self.insert_left(addr.clone(), hash, true) },
                    _ => panic!("didnt understand direction in join. expected RGT or LFT got {:?}", direction)
                }
            },
            _ => panic!("didnt understand {:?}", msg)
        }
    }

    /// first join request, everything else is called from here
    fn join_other(&mut self, entry: Addr) {
        let hash = calculate_hash(&entry);

        if hash > self.hash {
            self.join_other_right(entry, hash);
        } else if hash < self.hash {
            self.join_other_left(entry, hash);//TODO join left
        } else {
            panic!("Tried to join self");
        }
    }

    /// check and insert if node should be inserted right (if hash is bigger than right then go
    /// right)
    /// if not, sends it to right node to check for the same thing
    fn join_other_right(&mut self, to_join: Addr, hash: u64) {
        if hash == self.hash {
            panic!("attempting to join to self");
        }

        match &self.right_neighbour {
            Some(right_neighbour) => {
            if hash < right_neighbour.hash { //if node belongs here
                    self.insert_right(to_join, hash, false);
                } else if hash > right_neighbour.hash {
                    self.request_join_right(to_join);
                } else {
                    println!("[JOIN OTHER RIGHT] attempt to join same node, we good, stopping there");
                }
            }
            None => {
                self.insert_right(to_join, hash, false);
            }
        }
    }

    /// check and insert if node should be inserted left (if hash is bigger than self then go
    /// right)
    /// if not, sends it to right node to check for the same thing
    fn join_other_left(&mut self, to_join: Addr, hash: u64) {
        if hash == self.hash {
            panic!("attempting to join to self");
        }

        match &self.left_neighbour {
            Some(left_neighbour) => {
                if hash > left_neighbour.hash { //if node belongs here
                    self.insert_left(to_join, hash, false);
                } else if hash < left_neighbour.hash {
                    self.request_join_left(to_join);
                } else { 
                    println!("[JOIN OTHER LEFT] attempt to join same node, we good, stopping there");
                }
            }
            None => {
                self.insert_left(to_join, hash, false);
            }
        }
    }

    // insert a nodes on the left, and sends request to left node
    fn insert_left(&mut self, addr: Addr, hash: u64, join: bool) {
        match &mut self.left_neighbour {
            Some(neighbour) => {
                neighbour.hash = hash;
                neighbour.address = addr.clone();

                if !join {
                    self.send_join_left(addr.clone()); //send req to other left
                }
            },
            None => {
                self.left_neighbour = Some(Neighbour{ hash, address: addr.clone() });
                if !join {
                    self.send_join_left(addr.clone());
                }
            }
        }

        let src = self.address.clone();
        println!("[INSERT] {} is left of {}",
                 src.to_string(), addr.to_string());
    }

    fn insert_right(&mut self, addr: Addr, hash: u64, join: bool) {
        match &mut self.right_neighbour {//same ish match done before TODO
            Some(neighbour) => {
                neighbour.hash = hash;
                neighbour.address = addr.clone();

                if !join {
                    self.request_join_right(addr.clone()); //send req to other right
                }
            },
            None => {
                self.right_neighbour = Some(Neighbour{ hash, address: addr.clone() });
                if !join {
                    self.send_join_right(addr.clone());
                }
            }
        }

        let src = self.right_neighbour.clone().unwrap().address;
        println!("[INSERT] {} is right of {}",
                 src.to_string(), addr.to_string());
    }

    fn request_join_right(&self, addr: Addr) -> Option<()> {
        let dest = self.right_neighbour.clone()?.address;
        let msg = format!("REQINS:RGT:{}", addr.to_string());
        self.socket.send_to(msg.as_bytes(), dest.to_string()).unwrap();
        println!("Sending {:?} to {:?}:{:?}", msg, addr.ip, addr.port);
        Some(())
    }

    /// send a request to left node to insert it to its right
    /// this way is safer (nodes have to be reviewed by peer)
    /// other way would be to return address to other and tell 
    /// the requesing node to go insert itself
    fn request_join_left(&self, addr: Addr) -> Option<()> {
        let dest = self.left_neighbour.clone()?.address;
        let msg = format!("REQINS:LFT:{}", addr.to_string());
        self.socket.send_to(msg.as_bytes(), dest.to_string()).unwrap();
        println!("Sending {:?} to {:?}:{:?}", msg, addr.ip, addr.port);
        Some(())
    }

    /// sends a insertion request to right node, and sends old right node info to 
    /// joining node
    fn send_join_left(&self, to_join:Addr) {
        let lft_addr = self.left_neighbour.clone().unwrap().address.to_string();

        if self.left_neighbour.clone().unwrap().hash != calculate_hash(&to_join) { //do not send
                                                                                   //reqins to req
            let msg = format!("INS:LFT:{}", lft_addr.to_string());//msg to left neighbour
            self.socket.send_to(msg.as_bytes(), lft_addr.clone()).unwrap();
            println!("[SEND JOIN LEFT] sending {} to {}", msg, lft_addr);
        }

        //THEN sends info about new node (left then right)TODO
        //self.socket.send_to(msg.as_bytes(), to_join.to_string()).unwrap();
        let msg = format!("JOIN:{}:{}", "LFT", self.address.to_string());
        self.socket.send_to(msg.as_bytes(), to_join.to_string()).unwrap();
        println!("[SEND JOIN LEFT] sending {} to {}", msg, lft_addr);
    }

    fn send_join_right(&self, to_join:Addr) {
        let rgt_addr = self.right_neighbour.clone().unwrap().address.to_string();

        if self.right_neighbour.clone().unwrap().hash != calculate_hash(&to_join) { //do not send
                                                                                   //reqins to req
            let msg = format!("INS:LFT:{}", rgt_addr.to_string());//msg to left neighbour
            self.socket.send_to(msg.as_bytes(), rgt_addr.clone()).unwrap();
            println!("[SEND JOIN LEFT] sending {} to {}", msg, rgt_addr);
        }

        //THEN sends info to new node (left then right)
        // TODO RECHECK PLEASE
        let msg = format!("JOIN:{}:{}", "RGT", self.address.to_string());
        self.socket.send_to(msg.as_bytes(), to_join.to_string()).unwrap();
    }
}

fn calculate_hash<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}
