use std::io::Read;
use std::net::{IpAddr, SocketAddr};
use std::thread;
use std::time::Duration;

use rand::random;
use rate_limit::SyncLimiter;

use socket2::{Domain, Protocol, Socket, Type};

use pnet_packet::icmp::{self, echo_reply, echo_request, IcmpTypes};
use pnet_packet::ipv4::Ipv4Packet;
use pnet_packet::Packet;

pub fn ping(
    addr: IpAddr,
    timeout: Option<Duration>,
    ttl: Option<u32>,
    ident: Option<u16>,
    payload: &[u8],
) -> anyhow::Result<()> {
    let timeout = match timeout {
        Some(timeout) => Some(timeout),
        None => Some(Duration::from_secs(4)),
    };

    let dest = SocketAddr::new(addr, 0);

    let mut socket = Socket::new(Domain::IPV4, Type::RAW, Some(Protocol::ICMPV4))?;
    socket.set_ttl(ttl.unwrap_or(64))?;
    socket.set_write_timeout(timeout)?;

    let payload = b"hello world";
    let send_handler = thread::spawn(move || {
        let limiter = SyncLimiter::full(1, Duration::from_secs(1));

        let mut seq =1u16;
        loop {
            limiter.take();
            println!("Sending: {:?}", String::from_utf8_lossy(payload));

            let mut buf = vec![0; 8 + payload.len()]; // 8 bytes of header, then payload
            let mut packet = echo_request::MutableEchoRequestPacket::new(&mut buf[..]).unwrap();
            packet.set_icmp_type(icmp::IcmpTypes::EchoRequest);
            packet.set_identifier(ident.unwrap_or(random()));
            packet.set_sequence_number(seq);
            seq +=1;

            // let default_payload: &Token = &random();
            packet.set_payload(payload);

            let icmp_packet = icmp::IcmpPacket::new(packet.packet()).unwrap();
            let checksum = icmp::checksum(&icmp_packet);
            packet.set_checksum(checksum);

            match socket.send_to(&mut buf, &dest.into()) {
                Ok(n) => {}
                Err(e) => {
                    println!("Error in send: {:?}", e);
                }
            }
        }
    });

    let mut socket2 = Socket::new(Domain::IPV4, Type::RAW, Some(Protocol::ICMPV4))?;
    socket2.set_read_timeout(timeout)?;
    loop {
        let mut buffer: [u8; 2048] = [0; 2048];
        let size = match socket2.read(&mut buffer) {
            Ok(n) => n,
            Err(e) => {
                println!("Error in read: {:?}", e);
                continue;
            }
        };

        let buffer = &buffer[..size];

        let ipv4_packet = Ipv4Packet::new(&buffer).unwrap();
        let icmp_packet = pnet_packet::icmp::IcmpPacket::new(ipv4_packet.payload()).unwrap();

        if icmp_packet.get_icmp_type() != IcmpTypes::EchoReply
            || icmp_packet.get_icmp_code() != echo_reply::IcmpCodes::NoCode
        {
            continue;
        }

        let echo_replay = icmp::echo_reply::EchoReplyPacket::new(icmp_packet.packet()).unwrap();

        println!(
            "Reply: id={:?}, seq={:?}, payload={:?}",
            echo_replay.get_identifier(),
            echo_replay.get_sequence_number(),
            String::from_utf8_lossy(echo_replay.payload())
        );
    }

    send_handler.join().unwrap();

    return Ok(());
}
