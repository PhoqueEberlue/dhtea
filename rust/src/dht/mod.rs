mod req_listener;

use crossbeam_channel::{bounded, Receiver as Crossbeam_Receiver, TryRecvError};
use ctrlc;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::net::UdpSocket;
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::sync::Arc;
use std::thread;

#[derive(Hash, Clone)]
pub struct Addr {//TODO extends ipv4?
    pub ip: String,
    pub port: u32,
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
    address: Addr, //ok but more perf if port and addr dissociated
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

    /// runs a node until ctrl c
    pub fn run(&mut self, remote_address: Option<Addr>) -> std::io::Result<()> {
        // Cloning to pass the Atomic Reference Counted to the thread
        let socket_ref = self.socket.clone();

        // Creating a Multi Producer Single Consummer channel
        let (tx, rx) = mpsc::channel();
        let (shutdown_tx, shutdown_rx) = bounded(0); //shutdown channels

        ctrlc::set_handler(move || {
            //exit on ctlc
            println!("received Ctrl+C!");
            shutdown_tx.send(()).unwrap();
        })
        .expect("error in ctrlc");

        // Launching the thread that listens to the others nodes
        let mv_shutdown_rx = shutdown_rx.clone();

        let handle_listener = thread::spawn(move || {
            req_listener::run(socket_ref, tx, mv_shutdown_rx).unwrap();
        });

        self.run_req_handler(remote_address, rx, shutdown_rx)
            .unwrap();

        handle_listener.join().unwrap();

        Ok(())
    }

    // Main function of the request handler
    fn run_req_handler(
        &mut self,
        remote_address: Option<Addr>,
        channel_consummer: Receiver<String>,
        shutdown_rx: Crossbeam_Receiver<()>,
    ) -> std::io::Result<()> {
        // If remote_address is not None, unwraps into address
        if let Some(address) = remote_address {
            let msg = "INIT CONNECT";
            self.socket.send_to(msg.as_bytes(), address.to_string())?;
            println!("Sending {:?} to {:?}", msg, address.to_string());
        }

        loop {
            self.handle_request(&channel_consummer);
            match shutdown_rx.try_recv() {
                Ok(()) | Err(TryRecvError::Disconnected) => {
                    println!("stoping run req handler");
                    break;
                }
                Err(TryRecvError::Empty) => {}
            }
        }
        self.send_leave();
        Ok(())
    }

    fn handle_request(&mut self, channel_consummer: &Receiver<String>) {
        let rcv = channel_consummer.recv().unwrap(); //blocking : ctrl c stops here for now TODO
        let mut rcv = rcv.split(";"); //split at end of source; there is no ; in an address, we good
        let src = rcv.next().unwrap().replacen("SRC:", "", 1); //shadowing go brrrrrr}
        let msg = rcv.next().unwrap().replacen("MSG:", "", 1); //replace only the first occurence

        let mut src_splt = src.split(":");
        let ip = src_splt.next().unwrap().replacen(":", "", 1);
        let port = src_splt.next().unwrap().parse().unwrap();

        let addr: Addr = Addr { ip, port };
        let hash = calculate_hash(&addr);

        println!(
            "[HANDLE REQUEST] self {} req {} msg {}",
            self.hash, hash, msg
        );

        let mut msg = msg.split(":");

        match msg.next().unwrap() {
            "JOIN" => {
                let direction = msg.next().unwrap();

                let ip = msg.next().unwrap();
                let port = msg.next().unwrap();

                let addr = Addr {
                    ip: String::from(ip),
                    port: port.parse().unwrap(),
                };
                let hash = calculate_hash(&addr);

                match direction {
                    "LFT" => self.insert_left(addr, hash, true),
                    "RGT" => self.insert_right(addr, hash, true),
                    _ => panic!("join fail"),
                }
            }
            "INIT CONNECT" => {
                print!("[INIT CONNECT]");
                self.join(addr);
            }
            "REQINS" => {
                let direction = msg.next().unwrap();
                let ip = msg.next().unwrap();
                let port = msg.next().unwrap();
                let addr_joining = Addr {
                    ip: String::from(ip),
                    port: port.parse().unwrap(),
                }; //SHADOWING !!!

                let hash = calculate_hash(&addr_joining);
                println!(
                    "[REQINS] from {} joining {} hash {}",
                    addr.to_string(),
                    addr_joining.to_string(),
                    hash
                );

                match direction {
                    "RGT" => self.join_right(addr_joining, hash),
                    "LFT" => self.join_left(addr_joining, hash),
                    _ => panic!(
                        "didnt understand direction in join. expected rgt or lft got {:?}",
                        direction
                    ),
                }
            }
            _ => panic!("didnt understand {:?}", msg),
        }
    }

    /// first join request, everything else is called from here
    fn join(&mut self, entry: Addr) {
        let hash = calculate_hash(&entry);

        if hash > self.hash {
            self.join_right(entry, hash);
        } else if hash < self.hash {
            self.join_left(entry, hash);
        } else {
            panic!("Tried to join self");
        }
    }

    /// check and insert if node should be inserted right (if hash is bigger than right then go
    /// right)
    /// if not, sends it to right node to check for the same thing
    fn join_right(&mut self, to_join: Addr, hash: u64) {
        if hash == self.hash {
            panic!("attempting to join to self");
        }

        match &mut self.right_neighbour.clone() {
            Some(right_neighbour) => {
                if hash < right_neighbour.hash {
                    //if node belongs here
                    self.insert_right(to_join, hash, false);
                } else if hash > right_neighbour.hash {
                    self.reqins_right(to_join.clone());
                    if self.is_last_node() {
                        self.insert_right(to_join, hash, false);
                    }
                } else {
                    println!(
                        "[JOIN OTHER RIGHT] attempt to join same node, we good, stopping there"
                    );
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
    fn join_left(&mut self, to_join: Addr, hash: u64) {
        if hash == self.hash {
            panic!("attempting to join to self");
        }

        match &mut self.left_neighbour.clone() {
            Some(left_neighbour) => {
                if hash > left_neighbour.hash {
                    //if node belongs here
                    self.insert_left(to_join, hash, false);
                } else if hash < left_neighbour.hash {
                    self.reqins_left(to_join.clone()); // if not sends to left
                    if self.is_last_node() {
                        self.insert_left(to_join, hash, false); //INSERT ONLY AFTER REQINS
                                                                //if reqins uneeded, node will check
                                                                //and stop
                    }
                } else {
                    println!(
                        "[JOIN OTHER LEFT] attempt to join same node, we good, stopping there"
                    );
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
                let old = Some(neighbour.clone());
                neighbour.hash = hash;
                neighbour.address = addr.clone();

                if !join {
                    self.send_join_left(addr.clone(), old); //send req to other left
                }
            }
            None => {
                self.left_neighbour = Some(Neighbour {
                    hash,
                    address: addr.clone(),
                });

                if self.is_last_node() {
                    self.right_neighbour = Some(Neighbour {
                        hash,
                        address: addr.clone(),
                    });

                    println!(
                        "[INSERT] {} is right of {}",
                        self.left_neighbour.clone().unwrap().address.to_string(),
                        self.address.to_string()
                    );
                }
                if !join {
                    self.send_join_left(addr.clone(), None);
                }
            }
        }

        let src = self.left_neighbour.clone().unwrap().address;
        println!(
            "[INSERT] {} is left of {}",
            src.to_string(),
            self.address.to_string()
        );
    }

    fn insert_right(&mut self, addr: Addr, hash: u64, join: bool) {
        match &mut self.right_neighbour {
            Some(neighbour) => {
                let old = Some(neighbour.clone());
                neighbour.hash = hash;
                neighbour.address = addr.clone();

                if !join {
                    self.send_join_right(addr.clone(), old); //send req to other right
                }
            }
            None => {
                self.right_neighbour = Some(Neighbour {
                    hash,
                    address: addr.clone(),
                });

                if self.left_neighbour.is_none() {
                    self.left_neighbour = Some(Neighbour {
                        hash,
                        address: addr.clone(),
                    });

                    println!(
                        "[INSERT] {} is left of {}",
                        self.left_neighbour.clone().unwrap().address.to_string(),
                        self.address.to_string()
                    );
                }

                if !join {
                    self.send_join_right(addr.clone(), None);
                }
            }
        }

        let src = self.right_neighbour.clone().unwrap().address;
        println!(
            "[INSERT] {} is right of {}",
            src.to_string(),
            self.address.to_string()
        );
    }

    /// send a request to left node to insert it to its right
    /// this way is safer (nodes have to be reviewed by peer)
    /// other way would be to return address to other and tell
    /// the requesing node to go insert itself
    fn reqins_right(&self, addr: Addr) -> Option<()> {
        let dest = self.right_neighbour.clone()?.address;
        let msg = format!("REQINS:RGT:{}", addr.to_string());
        self.socket
            .send_to(msg.as_bytes(), dest.to_string())
            .unwrap();
        println!("Sending {:?} to {:?}", msg, dest.to_string());
        Some(())
    }

    /// send a request to left node to insert it to its right
    /// this way is safer (nodes have to be reviewed by peer)
    /// other way would be to return address to other and tell
    /// the requesing node to go insert itself
    fn reqins_left(&self, addr: Addr) -> Option<()> {
        let dest = self.left_neighbour.clone()?.address;
        let msg = format!("REQINS:LFT:{}", addr.to_string());
        self.socket
            .send_to(msg.as_bytes(), dest.to_string())
            .unwrap();
        println!("Sending {:?} to {:?}", msg, dest.to_string());
        Some(())
    }

    /// sends a insertion request to right node, and sends old right node info to
    /// joining node
    fn send_join_left(&self, to_join: Addr, old: Option<Neighbour>) {
        let lft_addr = self.left_neighbour.clone().unwrap().address.to_string();

        if old.is_some() {
            //do not send reqins to req
            let msg = format!("JOIN:{}:{}", "LFT", old.unwrap().address.to_string()); //msg to left neighbour
            self.socket
                .send_to(msg.as_bytes(), to_join.to_string())
                .unwrap();
            println!("[SEND JOIN LEFT] sending {} to {}", msg, lft_addr);
        }

        //THEN sends info about new node (left then right)
        //self.socket.send_to(msg.as_bytes(), to_join.to_string()).unwrap();
        let msg = format!("JOIN:{}:{}", "LFT", self.address.to_string());
        self.socket
            .send_to(msg.as_bytes(), to_join.to_string())
            .unwrap();
        println!("[SEND JOIN LEFT] sending {} to {}", msg, lft_addr);
    }

    fn send_join_right(&self, to_join: Addr, old: Option<Neighbour>) {
        let rgt_addr = self.right_neighbour.clone().unwrap().address.to_string();

        if old.is_some() {
            //do not send to req
            let msg = format!("JOIN:{}:{}", "LFT", old.unwrap().address.to_string()); //msg to left neighbour
            self.socket
                .send_to(msg.as_bytes(), to_join.to_string())
                .unwrap();
            println!("[SEND JOIN LEFT] sending {} to {}", msg, rgt_addr);
        }

        //THEN sends info to new node (left then right)
        let msg = format!("JOIN:{}:{}", "RGT", self.address.to_string());
        self.socket
            .send_to(msg.as_bytes(), to_join.to_string())
            .unwrap();
    }

    /// leaves the dht
    fn send_leave(&self) {
        if self.left_neighbour.is_some() && self.right_neighbour.is_some() {
            let lft_addr = self.left_neighbour.clone().unwrap().address.to_string();
            let rgt_addr = self.right_neighbour.clone().unwrap().address.to_string();

            println!(
                "[LEAVING] sending neighbour info to {} and {}",
                lft_addr, rgt_addr
            );

            //abusing join instead of doing a proper leave wont hurt
            //the progression in the future Clueless
            let lft_msg = format!("JOIN:{}:{}", "LFT", rgt_addr);
            let rgt_msg = format!("JOIN:{}:{}", "RGT", lft_addr);

            self.socket
                .send_to(lft_msg.as_bytes(), lft_addr.to_string())
                .unwrap();
            self.socket
                .send_to(rgt_msg.as_bytes(), rgt_addr.to_string())
                .unwrap();
        }
    }

    /// check if this is the last node of dht (as in if the circle werent closed, would that one
    /// be an extrimity)
    fn is_last_node(&self) -> bool {
        let mut res: bool = false;
        if self.left_neighbour.is_none() || self.right_neighbour.is_none() {
            res = true;
        } else if self.right_neighbour.clone().unwrap().hash < self.hash
            || self.left_neighbour.clone().unwrap().hash > self.hash
        {
            res = true;
        }
        println!("Last node of ring");
        res
    }
}

fn calculate_hash<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}
