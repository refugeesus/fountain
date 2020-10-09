use fountaincode::{
    decoder::{CatchResult, Decoder},
    encoder::{Encoder, EncoderType},
};
use rand::{distributions::Alphanumeric, thread_rng, Rng};

fn enc_dec_helper(
    total_len: usize,
    chunk_len: usize,
    loss: f32,
    enc_type: EncoderType,
) {
    let s: String = thread_rng()
        .sample_iter(Alphanumeric)
        .take(total_len)
        .collect();
    let buf = s.into_bytes();
    let len = buf.len();
    let to_compare = buf.clone();

    let mut enc = Encoder::robust(buf, chunk_len, enc_type);
    let mut dec = Decoder::new(len, chunk_len);

    let mut loss_rng = thread_rng();

    loop {
        if loss_rng.gen::<f32>() > loss {
            let drop = enc.drop();
            match dec.catch(drop) {
                CatchResult::Missing(stats) => {
                    //a systematic encoder and no loss on channel should only need k symbols
                    //assert_eq!(stats.cnt_chunks-stats.unknown_chunks, cnt_drops)
                    println!("Missing blocks {:?}", stats);
                }
                CatchResult::Finished(data, stats) => {
                    println!("Finished, stats: {:?}", stats);
                    assert_eq!(to_compare.len(), data.len());
                    for i in 0..len {
                        assert_eq!(to_compare[i], data[i]);
                    }
                    return;
                }
            }
        }
    }
}

#[test]
fn robust_enc_dec_simple_systemtatic() {
    enc_dec_helper(1_024, 512, 0.0, EncoderType::Systematic);
}

#[test]
fn robust_enc_dec_simple_random() {
    enc_dec_helper(1_024, 512, 0.0, EncoderType::Random);
}

#[test]
fn robust_enc_dec_uneven_sizes_systematic() {
    for size in 1000..1100 {
        for chunk in 100..130 {
            enc_dec_helper(size, chunk, 0.0, EncoderType::Systematic);
        }
    }
}

#[test]
fn robust_enc_dec_uneven_sizes_random() {
    for size in 1000..1100 {
        for chunk in 100..130 {
            enc_dec_helper(size, chunk, 0.0, EncoderType::Random);
        }
    }
}

#[test]
fn robust_enc_dec_simple_systemtatic_lossy() {
    enc_dec_helper(1_024, 512, 0.3, EncoderType::Systematic);
}

#[test]
fn robust_enc_dec_simple_random_lossy() {
    enc_dec_helper(1_024, 512, 0.3, EncoderType::Random);
}

#[test]
fn robust_enc_dec_uneven_sizes_systematic_lossy() {
    for size in 1000..1100 {
        for chunk in 100..130 {
            enc_dec_helper(size, chunk, 0.3, EncoderType::Systematic);
        }
    }
}

#[test]
fn robust_enc_dec_uneven_sizes_random_lossy() {
    for size in 1000..1100 {
        for chunk in 100..130 {
            enc_dec_helper(size, chunk, 0.3, EncoderType::Random);
        }
    }
}

#[test]
fn robust_enc_dec_combination_systematic_lossy() {
    for size in 1000..1100 {
        for chunk in 100..130 {
            for loss in &[0.1, 0.3, 0.5, 0.9] {
                enc_dec_helper(size, chunk, *loss, EncoderType::Systematic);
            }
        }
    }
}

#[test]
fn robust_enc_dec_combination_random_lossy() {
    for size in 1000..1100 {
        for chunk in 100..130 {
            for loss in &[0.1, 0.3, 0.5, 0.9] {
                enc_dec_helper(size, chunk, *loss, EncoderType::Random);
            }
        }
    }
}
