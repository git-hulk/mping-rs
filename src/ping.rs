use std::fmt;
use std::io::Read;
use std::net::{IpAddr, SocketAddr};
use std::time::Duration;

use rand::random;
use socket2::{Domain, Protocol, Socket, Type};

use crate::errors::Error;
use crate::packet::{EchoReply, EchoRequest, IcmpV4, IpV4Packet, ICMP_HEADER_SIZE};

const TOKEN_SIZE: usize = 24;
const ECHO_REQUEST_BUFFER_SIZE: usize = ICMP_HEADER_SIZE + TOKEN_SIZE;
type Token = [u8; TOKEN_SIZE];

pub fn ping(
    addr: IpAddr,
    timeout: Option<Duration>,
    ttl: Option<u32>,
    ident: Option<u16>,
    seq_cnt: Option<u16>,
    payload: Option<&Token>,
) -> Result<(), Error> {
    let timeout = match timeout {
        Some(timeout) => Some(timeout),
        None => Some(Duration::from_secs(4)),
    };

    let dest = SocketAddr::new(addr, 0);
    let mut buffer = [0; ECHO_REQUEST_BUFFER_SIZE];

    let default_payload: &Token = &random();

    let request = EchoRequest {
        ident: ident.unwrap_or(random()),
        seq_cnt: seq_cnt.unwrap_or(1),
        payload: payload.unwrap_or(default_payload),
    };

    if request.encode::<IcmpV4>(&mut buffer[..]).is_err() {
        return Err(Error::InternalError.into());
    }
    let mut socket = Socket::new(Domain::IPV4, Type::RAW, Some(Protocol::ICMPV4))?;

    socket.set_ttl(ttl.unwrap_or(64))?;

    socket.set_write_timeout(timeout)?;

    socket.send_to(&mut buffer, &dest.into())?;

    socket.set_read_timeout(timeout)?;

    loop {
        let mut buffer: [u8; 2048] = [0; 2048];
        let size = socket.read(&mut buffer)?;
        let buffer = &buffer[..size];

        let _reply = {
            let ipv4_packet = match IpV4Packet::decode(&buffer) {
                Ok(packet) => packet,
                Err(e) => {
                    println!("Error1: {}", e);
                    return Ok(());
                }
            };

            println!("ipv4_packet: {:?}", &buffer);


            let echo_reply = match EchoReply::decode::<IcmpV4>(ipv4_packet.data) {
                Ok(reply) => reply,
                Err(e) => {
                    println!("Error2: {}", e);
                    return Ok(());
                }
            };

            println!("Reply: {:?}", echo_reply.seq_cnt);
            break
        };
    }

    return Ok(());
}
