use std::net::UdpSocket;
use std::sync::mpsc::Sender;
use std::sync::Arc;

// Main function of the request listener
pub fn run(
    socket: Arc<UdpSocket>,
    channel_producer: Sender<String>,
) -> std::io::Result<()> {
    loop {
        let mut buf = Vec::with_capacity(2048);
        buf.resize(2048, 0);

        let (amt, src) = socket.recv_from(&mut buf)?;
        buf.truncate(amt);

        channel_producer
            .send(String::from_utf8(buf).unwrap())
            .unwrap();
    }
}
