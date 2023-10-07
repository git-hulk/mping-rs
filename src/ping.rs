use std::io::Read;
use std::net::{IpAddr, SocketAddr};
use std::thread;
use std::time::Duration;

use rand::random;
use rand::Rng;
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
    len: usize,
) -> anyhow::Result<()> {
    let timeout = match timeout {
        Some(timeout) => Some(timeout),
        None => Some(Duration::from_secs(5)),
    };

    let pid = ident.unwrap_or(random());


    let dest = SocketAddr::new(addr, 0);

    let socket = Socket::new(Domain::IPV4, Type::RAW, Some(Protocol::ICMPV4))?;
    socket.set_ttl(ttl.unwrap_or(64))?;
    socket.set_write_timeout(timeout)?;

    // println!("payload: {:?}", _payload);
    // let payload = b"hello world";

    let rand_payload = random_bytes(len);
    let read_rand_payload = rand_payload.clone();

    // send
    thread::spawn(move || {
        
        let zero_payload = vec![0; len];
        let one_payload = vec![1; len];
        let fivea_payload = vec![0x5A; len];

        let payloads: [&[u8]; 4] = [&rand_payload, &zero_payload, &one_payload, &fivea_payload];

        let limiter = SyncLimiter::full(1, Duration::from_millis(1000));
        let mut seq = 1u16;
        loop {
            limiter.take();

            let payload = payloads[seq as usize % payloads.len()];

            let mut buf = vec![0; 8 + payload.len()]; // 8 bytes of header, then payload
            let mut packet = echo_request::MutableEchoRequestPacket::new(&mut buf[..]).unwrap();
            packet.set_icmp_type(icmp::IcmpTypes::EchoRequest);
            packet.set_identifier(pid);
            packet.set_sequence_number(seq);
            seq += 1;

            packet.set_payload(&payload);

            let icmp_packet = icmp::IcmpPacket::new(packet.packet()).unwrap();
            let checksum = icmp::checksum(&icmp_packet);
            packet.set_checksum(checksum);

            match socket.send_to(&mut buf, &dest.into()) {
                Ok(_) => {}
                Err(e) => {
                    println!("Error in send: {:?}", e);
                    return;
                }
            }
        }
    });

    // read
    let zero_payload = vec![0; len];
    let one_payload = vec![1; len];
    let fivea_payload = vec![0x5A; len];

    let payloads: [&[u8]; 4] = [&read_rand_payload, &zero_payload, &one_payload, &fivea_payload];

    let mut socket2 = Socket::new(Domain::IPV4, Type::RAW, Some(Protocol::ICMPV4))?;
    socket2.set_read_timeout(timeout)?;

    let mut buffer: [u8; 2048] = [0; 2048];

    loop {
        let size = match socket2.read(&mut buffer) {
            Ok(n) => n,
            Err(e) => {
                if let Some(err_code) = e.raw_os_error() {
                    if err_code == 11 {
                        // { code: 11, kind: WouldBlock, message: "Resource temporarily unavailable" }
                        continue;
                    }
                }
                println!("Error in read: {:?}", &e);

                break;
            }
        };
        let buf = &buffer[..size];

        let ipv4_packet = Ipv4Packet::new(&buf).unwrap();
        let icmp_packet = pnet_packet::icmp::IcmpPacket::new(ipv4_packet.payload()).unwrap();

        if icmp_packet.get_icmp_type() != IcmpTypes::EchoReply
            || icmp_packet.get_icmp_code() != echo_reply::IcmpCodes::NoCode
        {
            continue;
        }

        let echo_replay = match icmp::echo_reply::EchoReplyPacket::new(icmp_packet.packet()) {
            Some(echo_replay) => echo_replay,
            None => {
                continue;
            }
        };

        if echo_replay.get_identifier() != pid {
            continue;
        }
        

        if payloads[echo_replay.get_sequence_number() as usize % payloads.len()]
            != echo_replay.payload()
        {
            println!("bitflip detected! seq={:?},", echo_replay.get_sequence_number());
        }

        println!(
            "Reply: id={:?}, seq={:?}, payload len={:?}",
            echo_replay.get_identifier(),
            echo_replay.get_sequence_number(),
            echo_replay.payload().len()
        );
    }

    Ok(())
}

fn random_bytes(len: usize) -> Vec<u8> {
    let mut rng = rand::thread_rng();
    let mut vec = vec![0u8; len];
    rng.fill(&mut vec[..]);

    vec
}
