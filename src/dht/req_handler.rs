use std::net::UdpSocket;
use std::sync::mpsc::Receiver;
use std::sync::Arc;

// Main function of the request handler
pub fn run(
    socket: Arc<UdpSocket>,
    remote_address: Option<String>,
    channel_consummer: Receiver<String>,
) -> std::io::Result<()> {
    // If remote_address is not None, unwraps into address
    if let Some(address) = remote_address {
        let msg = "Whats up";
        socket.connect(address)?;
        socket.send(msg.as_bytes())?;
    }

    loop {
        handle_request(&channel_consummer);
    }
}

fn handle_request(channel_consummer: &Receiver<String>) {
    let msg = channel_consummer.recv().unwrap();
    println!("{:?}", msg);
}
