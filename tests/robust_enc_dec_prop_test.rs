use fountaincode::{
    decoder::{CatchResult, Decoder},
    encoder::{Encoder, EncoderType},
};
use proptest::prelude::*;
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use stopwatch::Stopwatch;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10000))]
    #[test]
    fn robust_prop_test(total_len in 1024_usize..8192, chunk_len in 8_usize..512) {
        let s: String = thread_rng()
            .sample_iter(Alphanumeric)
            .take(total_len)
            .collect();
        let buf = s.into_bytes();
        let len = buf.len();
        let to_compare = buf.clone();

        let mut enc = Encoder::robust(buf, chunk_len, EncoderType::Systematic);
        let mut dec = Decoder::new(len, chunk_len);

        let sw = Stopwatch::start_new();
        let res = run(&mut enc, &mut dec);
        println!("time: {:#?}", sw.elapsed());

        prop_assert_eq!(to_compare, res);
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10000))]
    #[test]
    fn robust_prop_lossy_test(total_len in 1024_usize..8192, chunk_len in 8_usize..512) {
        let s: String = thread_rng()
            .sample_iter(Alphanumeric)
            .take(total_len)
            .collect();
        let buf = s.into_bytes();
        let len = buf.len();
        let to_compare = buf.clone();

        let mut dec = Decoder::new(len, chunk_len);
        let mut res: Vec<u8> = vec![];

        for loss in &[0.1, 0.3, 0.5, 0.9] {
            let mut enc = Encoder::robust(buf.clone(), chunk_len , EncoderType::Systematic);
            let sw = Stopwatch::start_new();
            res = run_lossy(&mut enc, &mut dec, *loss);
            println!("total_len: {:?}, chunk_len: {:?}, loss: {:?}, time: {:#?}", total_len, chunk_len, loss, sw.elapsed());
        }

        prop_assert_eq!(to_compare, res);
    }
}

fn run(enc: &mut Encoder, dec: &mut Decoder) -> Vec<u8> {
    loop {
        let drop = enc.drop();
        match dec.catch(drop) {
            CatchResult::Missing(_stats) => {
                // println!("Missing blocks {:?}", stats);
            }
            CatchResult::Finished(data, _stats) => {
                // println!("Finished, stats: {:?}", stats);
                break data;
            }
        }
    }
}

fn run_lossy(enc: &mut Encoder, dec: &mut Decoder, loss: f32) -> Vec<u8> {
    let mut loss_rng = thread_rng();
    loop {
        if loss_rng.gen::<f32>() > loss {
            let drop = enc.drop();
            match dec.catch(drop) {
                CatchResult::Missing(_stats) => {
                    // println!("Missing blocks {:?}", stats);
                }
                CatchResult::Finished(data, stats) => {
                    println!("overhead: {:?}", stats.overhead);
                    break data;
                }
            }
        }
    }
}
