use std::{io::Write, net::{TcpStream, UdpSocket}, time::Duration};

const SIM_TCP_ADDR: &str = "127.0.0.1:1740";
const SIM_UDP_ADDR: &str = "127.0.0.1:1110";
const DS_UDP_ADDR: &str = "127.0.0.1:64651";


fn main() -> std::io::Result<()> {
    let mut tcp = TcpStream::connect(SIM_TCP_ADDR)?;
    let udp = UdpSocket::bind(DS_UDP_ADDR)?;
    udp.connect(SIM_UDP_ADDR)?;

    let tcp_init = [
        0x00, 0x08, 0x02, 0x00, 0x00, 0xff, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x08, 0x02, 0x01, 0x00, 0xff, 0x00, 0x00, 0x00, 0x00, 0x00, 0x08,
        0x02, 0x02, 0x00, 0xff, 0x00, 0x00, 0x00, 0x00, 0x00, 0x08, 0x02, 
        0x03, 0x00, 0xff, 0x00, 0x00, 0x00, 0x00, 0x00, 0x08, 0x02, 0x04, 
        0x00, 0xff, 0x00, 0x00, 0x00, 0x00, 0x00, 0x08, 0x02, 0x05, 0x00, 
        0xff, 0x00, 0x00, 0x00, 0x00, 0x00, 0x06, 0x07, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x04, 0x0e, 0x41, 0x41, 0x41,
    ];

    tcp.write_all(&tcp_init)?;

    let mut sequence: u16 = 0x0000;
    loop {
        let sequence_bytes = sequence.to_be_bytes();

        let mut mode = (sequence / 10) % 6;

        if (sequence / 100) % 2 == 0 {
            mode = 0x80;
        }

        udp.send(&[sequence_bytes[0], sequence_bytes[1], 0x01, mode as u8, 0x11, 0x03])?;
        std::thread::sleep(Duration::from_millis(50));

        sequence += 1;
    }
}
