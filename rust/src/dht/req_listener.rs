use crossbeam_channel::{Receiver as Crossbeam_Receiver, TryRecvError};
use std::net::UdpSocket;
use std::sync::mpsc::Sender;
use std::sync::Arc;

// Main function of the request listener
/// main receive function
/// unwraps sockets and sends as channel producer
pub fn run(
    socket: Arc<UdpSocket>,
    channel_producer: Sender<String>,
    channel_stop: Crossbeam_Receiver<()>,
) -> std::io::Result<()> {
    loop {
        match channel_stop.try_recv() {
            Err(TryRecvError::Disconnected) => {
                println!("DISCONNECTED");
                break;
            }
            Ok(()) => {
                println!("DISCONNECTED");
                break;
            }
            Err(TryRecvError::Empty) => {}
        }

        let mut buf = Vec::with_capacity(2048);
        buf.resize(2048, 0);

        let (amt, src) = socket.recv_from(&mut buf)?;
        buf.truncate(amt);

        //send message and source to receiver
        let src: String = ["SRC:".to_string(), src.to_string()].join("");
        let msg: String = ["MSG:".to_string(), String::from_utf8(buf).unwrap()].join("");
        channel_producer.send([src, msg].join(";")).unwrap();
    }
    Ok(())
}
