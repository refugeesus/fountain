use std::fs::File;
use std::io::prelude::*;
use fountaincode::encoder::{Encoder, EncoderType};
use fountaincode::decoder::{Decoder, CatchResult::{Finished, Missing}};
use fountaincode::ldpc::droplet_decode;
use labrador_ldpc::LDPCCode;
use rand::{Rng, thread_rng};

fn enc_dec_helper(chunk_len: usize, loss: f32, enc_type: EncoderType, fname: &str) {
    let mut buf = Vec::new();
    let mut f = File::open(fname).unwrap();
    f.read_to_end(&mut buf).ok();
    let msg = buf.iter().map(|&c| c as char).collect::<String>();
    println!("{:?}", msg);
    let length = buf.len();
    let buf_org = buf.clone();

    //create an Encoder, and set the length of the chunks.
    let enc = Encoder::ideal(buf, chunk_len, enc_type);

    //create a Decoder
    let mut dec = Decoder::new(length, chunk_len);

    let mut loss_rng = thread_rng();
    //Encoder is exposed as Iterator
    for mut drop in enc {
        if loss_rng.gen::<f32>() > loss {
            //Decoder catches droplets
            println!("{:?}", drop);
            droplet_decode(&mut drop, LDPCCode::TM1280);
            match dec.catch(drop) {
                Missing(stats) => {
                    println!("{:?}", stats);
                }
                Finished(data, stats) => {
                    println!("Success: {:?}", stats);
                    for i in 0..length {
                        assert_eq!(buf_org[i], data[i]);
                    }
                    println!("Match: ");
                    let imp = data.iter().map(|&c| c as char).collect::<String>();
                    println!("{:?}", imp);
                    return;
                }
            }
        }
    }
}

#[test]
fn ldpc_test_enc_dec_simple() {
    enc_dec_helper(128, 0.0, EncoderType::RandLdpc(LDPCCode::TM1280, 0), "data/sample.txt");
}

#[test]
fn ldpc_test_enc_dec_systematic() {
    enc_dec_helper(128, 0.0, EncoderType::SysLdpc(LDPCCode::TM1280, 0), "data/sample.txt");
}

#[test]
fn ldpc_test_enc_dec_rand_loss() {
    for loss in &[0.05, 0.1, 0.2, 0.25, 0.3, 0.5, 0.9] {
        enc_dec_helper(128, *loss, EncoderType::RandLdpc(LDPCCode::TM1280, 0), "data/sample.txt");
    }
}

#[test]
fn ldpc_test_enc_dec_sys_loss() {
    for loss in &[0.05, 0.1, 0.2, 0.25, 0.3, 0.5, 0.9] {
        enc_dec_helper(128, *loss, EncoderType::SysLdpc(LDPCCode::TM1280, 0), "data/sample.txt");
    }
}
