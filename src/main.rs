use std::{
    net::{SocketAddr, UdpSocket},
    time::Duration,
};

use clap::Parser;
use hickory_proto::{
    op::{Message, MessageType, OpCode, Query},
    rr::{Name, RecordType},
    serialize::binary::{BinEncodable, BinEncoder},
};

#[derive(Parser)]
#[command(name = "resolve")]
#[command(about = "A simple to use DNS resolver")]
struct Cli {
    #[arg(long = "dns-server", default_value = "1.1.1.1")]
    dns_server: String,
    #[arg(long = "domain-name")]
    domain_name: String,
}

fn main() {
    let cli = Cli::parse();

    let domain_name = Name::from_ascii(cli.domain_name).unwrap();
    let dns_server: SocketAddr = format!("{}:53", cli.dns_server).parse().unwrap();

    let mut request_as_bytes: Vec<u8> = Vec::with_capacity(512);
    let mut response_as_bytes: Vec<u8> = vec![0; 512];

    let mut msg = Message::new();
    msg.set_id(rand::random::<u16>())
        .set_message_type(MessageType::Query)
        .add_query(Query::query(domain_name, RecordType::A))
        .set_op_code(OpCode::Query)
        .set_recursion_desired(true);

    let mut encoder = BinEncoder::new(&mut request_as_bytes);
    msg.emit(&mut encoder).unwrap();

    let localhost = UdpSocket::bind("0.0.0.0:0").expect("cannot bind to local socket");
    let timeout = Duration::from_secs(3);
    localhost.set_read_timeout(Some(timeout)).unwrap();
    localhost.set_nonblocking(false).unwrap();

    localhost
        .send_to(&request_as_bytes, dns_server)
        .expect("socket misconfigured");

    localhost
        .recv_from(&mut response_as_bytes)
        .expect("timeout reached");

    let dns_message = Message::from_vec(&response_as_bytes).expect("unable to parse response");

    dns_message
        .answers()
        .into_iter()
        .filter_map(|a| a.data())
        .filter_map(|resource| resource.as_a())
        .for_each(|v| println!("{}", v));
}
