use std::{net::{SocketAddr, TcpStream}, time::Duration};

fn main() {
    TcpStream::connect_timeout(&SocketAddr::from(([10, 88, 91, 2], 1740)), Duration::from_secs(5)).unwrap();
}