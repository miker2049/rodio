// use std::net::{TcpListener, TcpStream};

// fn handle_client(stream: TcpStream) {
//     // ...
//     print!("%",stream)
// }

// fn main() -> std::io::Result<()> {
//     let listener = TcpListener::bind("127.0.0.1:80")?;

//     // accept connections and process them serially
//     for stream in listener.incoming() {
//         handle_client(stream?);
//     }
//     Ok(())
// }



use rodio::source::Buffered;
use rosc::OscPacket;
use std::env;
use std::fs::File;
use std::io::BufReader;
use std::net::{SocketAddrV4, UdpSocket};
use std::collections::HashMap;
use std::str::FromStr;
use rodio::{Sink, Source};
extern crate rosc;

type PSample =  Buffered<rodio::Decoder<BufReader<File>>>;


fn main() {
    let (_stream, stream_handle) = rodio::OutputStream::try_default().unwrap();
    let sink = rodio::Sink::try_new(&stream_handle).unwrap();

    let mut buffs: HashMap<String, PSample > = HashMap::new();
    let args: Vec<String> = env::args().collect();
    println!("{:?}", args);
    let usage = format!("Usage {} IP:PORT", &args[0]);
    if args.len() < 2 {
        println!("{}", usage);
        ::std::process::exit(1)
    }
    let addr = match SocketAddrV4::from_str(&args[1]) {
        Ok(addr) => addr,
        Err(_) => panic!("{}",usage),
    };
    let sock = UdpSocket::bind(addr).unwrap();
    println!("Listening to {}", addr);

    let mut buf = [0u8; rosc::decoder::MTU];

    loop {
        match sock.recv_from(&mut buf) {
            Ok((size, addr)) => {
                println!("Received packet with size {} from: {}", size, addr);
                let packet = rosc::decoder::decode(&buf[..size]).unwrap();
                handle_packet(packet, &mut buffs, &sink);
            }
            Err(e) => {
                println!("Error receiving from socket: {}", e);
                break;
            }
        }
    }
}

fn handle_packet<'a>(packet: OscPacket,
                     buffs: & mut HashMap<String, PSample>,
                     sink: &Sink) {
    match packet {
        OscPacket::Message(msg) => {
            println!("OSC address: {}", msg.addr);
            println!("OSC arguments: {:?}", msg.args);
            match msg.addr.as_str() {
                "/read" => {
                    if let rosc::OscType::String(arg) = msg.args[0].clone() {
                        if let rosc::OscType::String(file) = msg.args[1].clone() {
                            match File::open(file) {
                                Ok(file) => {
                                    let buf=rodio::Decoder::new(
                                        BufReader::new(file)
                                    ).unwrap().buffered();
                                    buffs.insert(arg, buf);
                                },
                                Err(_) => println!("file not found!"),
                            }
                        }
                    }

                }
                "/play" => {
                    if let rosc::OscType::String(arg) = &msg.args[0]{
                        match buffs.get(arg)  {
                            Some(f) => {
                                // let resu = f.to_owned();
                                sink.append(f.to_owned());
                                // sink.append(f as &dyn rodio::Source);
                            },
                            None => println!("no found"),
                        }
                    }
                }
                _ => {
                    println!("{}", msg.addr)
                }
            }
        }
        OscPacket::Bundle(bundle) => {
            println!("OSC Bundle: {:?}", bundle);
        }
    }
}
