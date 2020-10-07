use std::fs::File;
use std::io::prelude::*;
use fountaincode::encoder::{Encoder, EncoderType};
use fountaincode::decoder::{Decoder, CatchResult::{Finished, Missing}};
use fountaincode::ldpc::{droplet_decode, DecoderType};
use labrador_ldpc::LDPCCode;
use hamming;

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
    let mut bers: Vec<u64> = Vec::new();
    //Encoder is exposed as Iterator
    for mut drop in enc {
        //Decoder catches droplets
        drop.data[2] ^=  1<<7 | 1<<5 | 1<<3;
        drop.data[7] ^=  1<<7 | 1<<5 | 1<<3;
        drop.data[8] ^=  1<<7 | 1<<5 | 1<<3;
        drop.data[3] ^=  1<<7 | 1<<5 | 1<<3;
        let org_drop = drop.data.clone();
        match decoder_type {
            DecoderType::Ms => {
                droplet_decode(&mut drop, LDPCCode::TM1280, DecoderType::Ms);
            }
            DecoderType::Bf => {
                droplet_decode(&mut drop, LDPCCode::TM1280, DecoderType::Bf);
            }
        }
        // Re-encode decoded codeword to compare with received code
        let new_code = LDPCCode::TM1280;
        let mut recode = vec![0u8; new_code.n() / 8];
        let mut data_copy = drop.data.clone();
        data_copy.resize(chunk_len, 0);
        new_code.copy_encode(&data_copy, &mut recode);

        // Sum XOR of two encoded messages
        let mut diffs: Vec<u8> = Vec::new();
        for i in 0..org_drop.len() {
            diffs.push(recode[i] ^ org_drop[i]);
        }
        bers.push(hamming::weight(diffs.as_ref()));

        match dec.catch(drop) {
            Missing(_stats) => {
                ()
            }
            Finished(data, stats) => {
                let mut avg: f64 = 0.0;
                for b in &bers {
                    avg += *b as f64;
                }
                avg = avg/bers.len() as f64;
                println!("Success: {:?} | Loss: {:?} | EncodeType: {:?} | Decoder Type: {:?} | AVG_BER: {:?}",
                    stats, loss, tempenctype, decoder_type, avg);
                assert_eq!(buf_org.len(), data.len());
                for i in 0..length {
                    assert_eq!(buf_org[i], data[i]);
                }
                return;
            }
        }
    }
}

#[test]
fn ber_test_enc_dec_systematic() {
    enc_dec_helper(128, 0.0, EncoderType::SysLdpc(LDPCCode::TM1280, 0), "data/sample2.txt", DecoderType::Bf);
}
