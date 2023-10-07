use std::io::Read;
use std::net::{IpAddr, SocketAddr};
use std::time::Duration;

use rand::random;

use socket2::{Domain, Protocol, Socket, Type};

use pnet_packet::icmp::{self, IcmpTypes, echo_request,echo_reply};
use pnet_packet::ipv4::Ipv4Packet;
use pnet_packet::Packet;

pub fn ping(
    addr: IpAddr,
    timeout: Option<Duration>,
    ttl: Option<u32>,
    ident: Option<u16>,
    payload:  &[u8],
) -> anyhow::Result<()> {
    let timeout = match timeout {
        Some(timeout) => Some(timeout),
        None => Some(Duration::from_secs(4)),
    };

    let dest = SocketAddr::new(addr, 0);

    // let mut buffer = [0; ECHO_REQUEST_BUFFER_SIZE];

    // let default_payload: &Token = &random();

    // let request = EchoRequest {
    //     ident: ident.unwrap_or(random()),
    //     seq_cnt: seq_cnt.unwrap_or(1),
    //     payload: payload.unwrap_or(default_payload),
    // };

    // if request.encode::<IcmpV4>(&mut buffer[..]).is_err() {
    //     return Err(Error::InternalError.into());
    // }

    

    let mut buf = vec![0; 8 + payload.len()]; // 8 bytes of header, then payload
    let mut packet = echo_request::MutableEchoRequestPacket::new(&mut buf[..]).unwrap();
    packet.set_icmp_type(icmp::IcmpTypes::EchoRequest);
    packet.set_identifier(ident.unwrap_or(random()));
    packet.set_sequence_number(1);

    // let default_payload: &Token = &random();
    packet.set_payload(b"hello world");

    let icmp_packet = icmp::IcmpPacket::new(packet.packet()).unwrap();
    let checksum = icmp::checksum(&icmp_packet);
    packet.set_checksum(checksum);

    let mut socket = Socket::new(Domain::IPV4, Type::RAW, Some(Protocol::ICMPV4))?;

    socket.set_ttl(ttl.unwrap_or(64))?;

    socket.set_write_timeout(timeout)?;

    socket.send_to(&mut buf, &dest.into())?;

    socket.set_read_timeout(timeout)?;

    loop {
        let mut buffer: [u8; 2048] = [0; 2048];
        let size = socket.read(&mut buffer)?;
        let buffer = &buffer[..size];

        let _reply = {
            // let ipv4_packet = match IpV4Packet::decode(&buffer) {
            //     Ok(packet) => packet,
            //     Err(e) => {
            //         return Err(e.into());
            //     }
            // };

            let ipv4_packet = Ipv4Packet::new(&buffer).unwrap();
            let icmp_packet = pnet_packet::icmp::IcmpPacket::new(ipv4_packet.payload()).unwrap();
            
            if icmp_packet.get_icmp_type() != IcmpTypes::EchoReply
                || icmp_packet.get_icmp_code() != echo_reply::IcmpCodes::NoCode
            {
                continue;
            }


            let echo_replay = icmp::echo_reply::EchoReplyPacket::new(icmp_packet.packet()).unwrap();

            println!("Reply: id={:?}, seq={:?}, payload={:?}",echo_replay.get_identifier(),echo_replay.get_sequence_number(), String::from_utf8_lossy(echo_replay.payload()));
            break;
        };
    }

    return Ok(());
}
