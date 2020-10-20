use tokio::net::UdpSocket;
use std::net::SocketAddr;
use std::fs::File;
use std::io::prelude::*;
use fountaincode::droplet::Droplet;
use fountaincode::encoder::{Encoder, EncoderType};
use fountaincode::decoder::{Decoder, CatchResult::{Finished, Missing}};
use fountaincode::ldpc::{droplet_decode, DecoderType};
use labrador_ldpc::LDPCCode;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use bincode;

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Payload {
    id: u8,
    len: usize,
    data: Droplet,
}

struct Server {
    socket: UdpSocket,
    buf: Vec<u8>,
    rcvd: Option<(usize, SocketAddr)>,
}

impl Server {
    async fn run(self, buckets: &mut HashMap<u8, Decoder>) -> Result<(), Box<dyn std::error::Error>> {
        let Server {
            mut socket,
            mut buf,
            mut rcvd,
        } = self;
        let mut bucket_status: HashMap<u8, String> = HashMap::new();
        loop {
            // First we check to see if there's a message we need to echo back.
            // If so then we try to send it back to the original source, waiting
            // until it's writable and we're able to do so.
            if let Some((_size, peer)) = rcvd {
                let mut temp: Payload = bincode::deserialize(&buf.as_slice())?;
                temp.data.data[2] ^=  1<<7 | 1<<5 | 1<<3;
                temp.data.data[7] ^=  1<<7 | 1<<5 | 1<<3;
                temp.data.data[8] ^=  1<<7 | 1<<5 | 1<<3;
                temp.data.data[3] ^=  1<<7 | 1<<5 | 1<<3;

                droplet_decode(&mut temp.data, LDPCCode::TC512, DecoderType::Ms);
                match buckets.get_mut(&temp.id) {
                    Some(dec) => {
                        match dec.catch(temp.data) {
                            Missing(_stats) => {
                                //println!("Stream: {:?} | {:?}", &temp.id, stats);
                                bucket_status.insert(temp.id, "working".to_string());
                            }
                            Finished(data, stats) => {
                                println!("Stream: {} | Success: {:?}", temp.id, stats);
                                bucket_status.insert(temp.id, "complete".to_string());
                                let message = data.iter().map(|&c| c as char).collect::<String>();
                                println!("{:?}", message);
                                for (k, v) in bucket_status.iter() {
                                    println!("Stream: {} | Status: {}", k,v);
                                }
                            }
                        }
                    }
                    None => {
                        println!("Make new decoder bucket ID: {} | BufLen: {}", temp.id, temp.len);
                        let mut dec = Decoder::new(temp.len as usize, 32);
                        match dec.catch(temp.data) {
                            Missing(_stats) => {
                                //println!("Stream: {:?} | {:?}", &temp.id, stats);
                            }
                            Finished(data, stats) => {
                                println!("Stream: {} | Success: {:?}", temp.id, stats);
                                let message = data.iter().map(|&c| c as char).collect::<String>();
                                println!("{:?}", message);
                                for (k, v) in bucket_status.iter() {
                                    println!("Stream: {} | Status: {}", k,v);
                                }
                            }
                        }
                        buckets.insert(temp.id, dec);
                        bucket_status.insert(temp.id, "working".to_string());
                    }
                }
                let status = bucket_status.get(&temp.id).unwrap();
                let response = bincode::serialize(&status)?;
                let _amt = socket.send_to(&response, &peer).await?;
            }
            // If we're here then `to_send` is `None`, so we take a look for the
            // next message we're going to echo back.
            rcvd = Some(socket.recv_from(&mut buf).await?);
        }
    }
}

struct Client {
    socket_addr: SocketAddr,
    id: u8,
    fname: String,
}

impl Client {
    async fn run(self, remote_addr: SocketAddr) -> Result<(), Box<dyn std::error::Error>> {
        let mut socket = UdpSocket::bind(self.socket_addr).await?;

        let mut buf = Vec::new();
        let chunk_len = 32;
        let mut f = File::open(self.fname).unwrap();
        f.read_to_end(&mut buf).ok();
        let buflen = buf.len();

        let enc = Encoder::robust(buf, chunk_len, EncoderType::SysLdpc(LDPCCode::TC512, 0));
        socket.connect(&remote_addr).await?;

        let mut incr: usize = 0;
        for drop in enc {
            let pl = Payload {
                id: self.id,
                len: buflen,
                data: drop,
            };

            let data = bincode::serialize(&pl)?;
            socket.send(&data).await?;

            let mut ret_buf = vec![0u8; 1024];
            socket.recv(&mut ret_buf).await?;
            let response: String = bincode::deserialize(&ret_buf)?;
            match response.as_ref() {
                "complete" => {
                    return Ok(());
                }
                "working" => {
                    incr += 1;
                    if incr > 700 {
                        println!("Over 700 drop error");
                        return Err("Sent over 700 droplets with no success response... exiting".into());
                    }
                },
                _ => {
                    return Err("Some unknown return value from server".into());
                }
            }
        }
        Ok(())
    }
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    let addr = "127.0.0.1:8080".to_string();
    let socket = UdpSocket::bind(&addr).await.unwrap();
    println!("Listening on: {}", socket.local_addr().unwrap());
    let mut buckets: HashMap<u8, Decoder> = HashMap::new();
    let server = Server {
        socket,
        buf: vec![0; 1024],
        rcvd: None,
    };

    // This starts the server task.
    let aggregator = tokio::spawn(async move {
        server.run(&mut buckets).await.unwrap()
    });
    let mut clients: Vec<Client> = Vec::new();
    let client1 = Client {
        socket_addr: "0.0.0.0:0".parse().unwrap(),
        id: 1,
        fname: "../data/sample.txt".to_string(),
    };

    let client2 = Client {
        socket_addr: "0.0.0.0:0".parse().unwrap(),
        id: 2,
        fname: "../data/sample2.txt".to_string(),
    };

    let client3 = Client {
        socket_addr: "0.0.0.0:0".parse().unwrap(),
        id: 3,
        fname: "../data/sample3.txt".to_string(),
    };
    clients.push(client1);
    clients.push(client2);
    clients.push(client3);
    for c in clients {
        tokio::spawn(async move {
            c.run("127.0.0.1:8080".parse().unwrap()).await.unwrap()
        });
    }
    if aggregator.await.is_ok() {
        Ok(())
    } else {
        Err("Return of server was not ok".into())
    }
}
