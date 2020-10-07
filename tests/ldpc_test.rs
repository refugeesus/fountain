use std::fs::File;
use std::io::prelude::*;
use fountaincode::encoder::{Encoder, EncoderType};
use fountaincode::decoder::{Decoder, CatchResult::{Finished, Missing}};
use fountaincode::ldpc::{droplet_decode, DecoderType};
use labrador_ldpc::LDPCCode;
use rand::{Rng, thread_rng};

fn enc_dec_helper(chunk_len: usize, loss: f64, enc_type: EncoderType, fname: &str, decoder_type: DecoderType) {
    let mut buf = Vec::new();
    let mut f = File::open(fname).unwrap();
    f.read_to_end(&mut buf).ok();
    let length = buf.len();
    let buf_org = buf.clone();
    let tempenctype = enc_type.clone();
    //create an Encoder, and set the length of the chunks.
    let enc = Encoder::ideal(buf, chunk_len, enc_type);
    //let enc = Encoder::robust(buf, chunk_len, enc_type, 0.2, None, 0.05);
    //create a Decoder
    let mut dec = Decoder::new(length, chunk_len);

    let mut loss_rng = thread_rng();
    //Encoder is exposed as Iterator
    let mut packets_lost: u32 = 0;
    let mut drops: u32 = 0;
    for mut drop in enc {
        drops += 1;
        if loss_rng.gen::<f64>() > loss {
            //Decoder catches droplets
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
                    let per = packets_lost as f32 / drops as f32;
                    println!("Success: {:?} | Loss: {:?} | EncodeType: {:?} | Decoder Type: {:?} | Lost: {:?} | Total: {:?} | Percentage: {:?}",
                        stats, loss, tempenctype, decoder_type, packets_lost, drops, per);
                    assert_eq!(buf_org.len(), data.len());
                    for i in 0..length {
                        assert_eq!(buf_org[i], data[i]);
                    }
                    return;
                }
            }
        } else {
            if loss_rng.gen::<f64>() > 0.5 {
                packets_lost += 1;
            } else {
                drop.data[2] ^=  1<<7 | 1<<5 | 1<<3;
                drop.data[7] ^=  1<<7 | 1<<5 | 1<<3;
                drop.data[8] ^=  1<<7 | 1<<5 | 1<<3;
                drop.data[3] ^=  1<<7 | 1<<5 | 1<<3;
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
                        let per = packets_lost as f32 / drops as f32;
                        println!("Success: {:?} | Loss: {:?} | EncodeType: {:?} | Decoder Type: {:?} | Lost: {:?} | Total: {:?} | Percentage: {:?}",
                            stats, loss, tempenctype, decoder_type, packets_lost, drops, per);
                        assert_eq!(buf_org.len(), data.len());
                        for i in 0..length {
                            assert_eq!(buf_org[i], data[i]);
                        }
                        return;
                    }
                }
            }
        }
    }
}


#[test]
fn ldpc_test_enc_dec_random() {
    enc_dec_helper(128, 0.0, EncoderType::RandLdpc(LDPCCode::TM1280, 0), "data/sample2.txt", DecoderType::Bf);

}

#[test]
fn ldpc_test_enc_dec_systematic() {
    enc_dec_helper(128, 0.1, EncoderType::SysLdpc(LDPCCode::TM1280, 0), "data/sample2.txt", DecoderType::Bf);
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

#[test]
fn ldpc_test_enc_dec_random_minsum() {
    enc_dec_helper(128, 0.0, EncoderType::RandLdpc(LDPCCode::TM1280, 0), "data/sample.txt", DecoderType::Ms);
}

#[test]
fn ldpc_test_enc_dec_systematic_minsum() {
    enc_dec_helper(128, 0.0, EncoderType::SysLdpc(LDPCCode::TM1280, 0), "data/sample.txt", DecoderType::Ms);
}

#[test]
fn ldpc_test_enc_dec_random_lossy_minsum() {
    for loss in &[0.05, 0.1, 0.2, 0.25, 0.3, 0.5, 0.9] {
        enc_dec_helper(128, *loss, EncoderType::RandLdpc(LDPCCode::TM1280, 0), "data/sample.txt", DecoderType::Ms);
    }
}

#[test]
fn ldpc_test_enc_dec_systematic_lossy_minsum() {
    for loss in &[0.05, 0.1, 0.2, 0.25, 0.3, 0.5, 0.9] {
        enc_dec_helper(128, *loss, EncoderType::SysLdpc(LDPCCode::TM1280, 0), "data/sample.txt", DecoderType::Ms);
    }
}

