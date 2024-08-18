use driverstation::traits::Bytes;
use std::{
    net::{SocketAddr, TcpStream, UdpSocket},
    time::Duration,
};

fn main() {
    let rx = UdpSocket::bind(SocketAddr::from(([0, 0, 0, 0], 1150)))
        .expect("unable to bind rx UDP socket");

    let tx = UdpSocket::bind(SocketAddr::from(([0, 0, 0, 0], 6789)))
        .expect("unable to bind tx UDP socket");
    tx.connect(SocketAddr::from(([10, 88, 91, 2], 1110)))
        .expect("unable to connect tx UDP socket");
    let mut buf = vec![];
    driverstation::send::udp::Packet::default().write_bytes(&mut buf);
    tx.send(&buf).expect("unable to send UDP packet");

    let mut buf = [0; 100];
    rx.recv(&mut buf)
        .expect("unable to revieve UDP return packet");

    TcpStream::connect_timeout(
        &SocketAddr::from(([10, 88, 91, 2], 1740)),
        Duration::from_secs(5),
    )
    .unwrap();
}
