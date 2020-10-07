use std::fs::File;
use std::io::prelude::*;
use fountaincode::encoder::{Encoder, EncoderType};
use fountaincode::decoder::{Decoder, CatchResult::{Finished, Missing}};
use fountaincode::ldpc::{droplet_decode, DecoderType};
use labrador_ldpc::LDPCCode;
use rand::{Rng, thread_rng};

fn enc_dec_helper(chunk_len: usize, loss: f32, enc_type: EncoderType, fname: &str, decoder_type: DecoderType) {
    let mut buf = Vec::new();
    let mut f = File::open(fname).unwrap();
    f.read_to_end(&mut buf).ok();
    let length = buf.len();
    let buf_org = buf.clone();
    let tempenctype = enc_type.clone();
    //create an Encoder, and set the length of the chunks.
    let enc = Encoder::ideal(buf, chunk_len, enc_type);

    //create a Decoder
    let mut dec = Decoder::new(length, chunk_len);

    let mut loss_rng = thread_rng();
    //Encoder is exposed as Iterator
    for mut drop in enc {
        if loss_rng.gen::<f32>() > loss {
            //Decoder catches droplets

            // Corrupt some bits
            drop.data[20] ^=  1<<7 | 1<<5 | 1<<3;
            drop.data[7] ^=  1<<7 | 1<<5 | 1<<3;

            match decoder_type {
                DecoderType::Ms => {
                    droplet_decode(&mut drop, LDPCCode::TM1280, DecoderType::Ms);
                }
                DecoderType::Bf => {
                    droplet_decode(&mut drop, LDPCCode::TM1280, DecoderType::Bf);
                }
            }
            match dec.catch(drop) {
                Missing(_stats) => {
                    ()
                }
                Finished(data, stats) => {
                    println!("Success: {:?} | Loss: {:?} | EncodeType: {:?} | Decoder Type: {:?}", stats, loss, tempenctype, decoder_type);
                    for i in 0..length {
                        assert_eq!(buf_org[i], data[i]);
                    }
                    return;
                }
            }
        }
    }
}

#[test]
fn ldpc_test_enc_dec_random() {
    enc_dec_helper(128, 0.0, EncoderType::RandLdpc(LDPCCode::TM1280, 0), "data/sample.txt", DecoderType::Ms);
}

#[test]
fn ldpc_test_enc_dec_systematic() {
    enc_dec_helper(128, 0.0, EncoderType::SysLdpc(LDPCCode::TM1280, 0), "data/sample.txt", DecoderType::Bf);
}

#[test]
fn ldpc_test_enc_dec_random_lossy() {
    for loss in &[0.05, 0.1, 0.2, 0.25, 0.3, 0.5, 0.9] {
        enc_dec_helper(128, *loss, EncoderType::RandLdpc(LDPCCode::TM1280, 0), "data/sample.txt", DecoderType::Bf);
    }
}

#[test]
fn ldpc_test_enc_dec_systematic_lossy() {
    for loss in &[0.05, 0.1, 0.2, 0.25, 0.3, 0.5, 0.9] {
        enc_dec_helper(128, *loss, EncoderType::SysLdpc(LDPCCode::TM1280, 0), "data/sample.txt", DecoderType::Bf);
    }
}
