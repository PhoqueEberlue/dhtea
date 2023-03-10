use std::net::UdpSocket;
use std::thread;

pub struct App_interface {
    socket_in: Arc<UdpSocket>,
    port: u32
}

impl App_interface {
    /// create sockets used for application communication
    pub fn new(port: u32) -> Self{
        let socket = Arc::new(match UdpSocket::bind(&local_address) {
            Ok(s) => s,
            Err(e) => panic!("{}", e),
        });
        
        App_interface{
            socket,
            port
        }
    }

    /// runs 
    pub fn run() {
        let thread = thread::spawn(move ||
           {

           })
    }

    fn socket_handler() {

    }
}
