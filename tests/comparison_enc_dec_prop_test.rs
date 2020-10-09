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
    fn compare_prop_test(total_len in 1024_usize..8192, chunk_len in 8_usize..512) {
        let s: String = thread_rng()
            .sample_iter(Alphanumeric)
            .take(total_len)
            .collect();
        let buf = s.into_bytes();
        let len = buf.len();

        let mut dec = Decoder::new(len, chunk_len );

        let mut renc = Encoder::robust(buf.clone(), chunk_len , EncoderType::Systematic);
        let mut sw = Stopwatch::start_new();
        let res1 = robust_run(&mut renc, &mut dec);
        let t1 = sw.elapsed();
        let mut ienc = Encoder::ideal(buf.clone(), chunk_len , EncoderType::Systematic);
        sw.restart();
        let res2 = ideal_run(&mut ienc, &mut dec);
        let t2 = sw.elapsed();
        println!("total_len: {:?}, chunk_len: {:?}, robust_time: {:?}, ideal_time: {:?}",
            total_len, chunk_len, t1, t2);

        prop_assert_eq!(&buf, &res1);
        prop_assert_eq!(&buf, &res2);
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10000))]
    #[test]
    fn compare_prop_lossy_test(total_len in 1024_usize..8192, chunk_len in 8_usize..512) {
        let s: String = thread_rng()
            .sample_iter(Alphanumeric)
            .take(total_len )
            .collect();
        let buf = s.into_bytes();
        let len = buf.len();

        let mut dec = Decoder::new(len, chunk_len );
        let mut res1: Vec<u8> = vec![];
        let mut res2: Vec<u8> = vec![];

        for loss in &[0.1, 0.3, 0.5, 0.9] {
            let mut renc = Encoder::robust(buf.clone(), chunk_len , EncoderType::Systematic);
            let mut sw = Stopwatch::start_new();
            res1 = robust_run_lossy(&mut renc, &mut dec, *loss);
            let t1 = sw.elapsed();
            let mut ienc = Encoder::ideal(buf.clone(), chunk_len, EncoderType::Systematic);
            sw.restart();
            res2 = ideal_run_lossy(&mut ienc, &mut dec, *loss);
            let t2 = sw.elapsed();
            println!("total_len: {:?}, chunk_len: {:?}, loss: {:?}, robust_time: {:?}, ideal_time: {:?}",
                total_len, chunk_len, loss, t1, t2);
        }

        prop_assert_eq!(&buf, &res1);
        prop_assert_eq!(&buf, &res2);
    }
}

fn robust_run(enc: &mut Encoder, dec: &mut Decoder) -> Vec<u8> {
    loop {
        let drop = enc.drop();
        match dec.catch(drop) {
            CatchResult::Missing(stats) => {
                println!("robust unknown_chunks: {:?}", stats.unknown_chunks);
            }
            CatchResult::Finished(data, stats) => {
                println!("robust_overhead: {:?}", stats.overhead);
                break data;
            }
        }
    }
}

fn ideal_run(enc: &mut Encoder, dec: &mut Decoder) -> Vec<u8> {
    loop {
        let drop = enc.drop();
        match dec.catch(drop) {
            CatchResult::Missing(stats) => {
                println!("ideal unknown_chunks: {:?}", stats.unknown_chunks);
            }
            CatchResult::Finished(data, stats) => {
                println!("ideal_overhead: {:?}", stats.overhead);
                break data;
            }
        }
    }
}

fn robust_run_lossy(enc: &mut Encoder, dec: &mut Decoder, loss: f32) -> Vec<u8> {
    let mut loss_rng = thread_rng();

    loop {
        if loss_rng.gen::<f32>() > loss {
            let drop = enc.drop();
            match dec.catch(drop) {
                CatchResult::Missing(_stats) => {
                    // println!("Missing blocks {:?}", stats);
                }
                CatchResult::Finished(data, stats) => {
                    println!("robust_overhead: {:?}", stats.overhead);
                    break data;
                }
            }
        }
    }
}

fn ideal_run_lossy(enc: &mut Encoder, dec: &mut Decoder, loss: f32) -> Vec<u8> {
    let mut loss_rng = thread_rng();
    loop {
        if loss_rng.gen::<f32>() > loss {
            let drop = enc.drop();
            match dec.catch(drop) {
                CatchResult::Missing(_stats) => {
                    // println!("Missing blocks {:?}", stats);
                }
                CatchResult::Finished(data, stats) => {
                    println!("ideal_overhead: {:?}", stats.overhead);
                    break data;
                }
            }
        }
    }
}
