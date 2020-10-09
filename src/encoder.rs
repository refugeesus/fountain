use crate::{
    droplet::{DropType, Droplet},
    soliton::Soliton,
    xor::xor_bytes,
};
use rand::{
    distributions::{Distribution, Uniform},
    rngs::StdRng,
    {Rng, SeedableRng},
};
use std::cmp;
use labrador_ldpc::LDPCCode;
use crate::ldpc::droplet_encode;
/// Encoder for Luby Transform codes.
///
/// In case you send the packages over UDP, the blocksize should be
/// the MTU size.
///
/// There are two encoding modes, Systematic and Random.  The
/// Systematic encoder first produces a set of the source
/// symbols. After each symbol is sent once, it switches to Random.
///
/// # Example
///
/// ```
/// use fountaincode::encoder::{Encoder, EncoderType};
///
/// // For demonstration purposes, our message is just a range `u8`s.
/// let msg: Vec<u8> = (0..255).collect();
///
/// let mut enc = Encoder::robust(msg, 64, EncoderType::Random);
///
/// for i in 1..10 {
///     println!("droplet {:?}: {:?}", i, enc.next());
/// }
/// ```
#[derive(Clone)]
pub struct Encoder {
    data: Vec<u8>,
    len: usize,
    blocksize: usize,
    rng: StdRng,
    dist: Uniform<usize>,
    cnt_blocks: usize,
    sol: Soliton,
    pub cnt: usize,
    encodertype: EncoderType,
}

impl Encoder {
    pub fn robust(
        data: Vec<u8>,
        blocksize: usize,
        encodertype: EncoderType,
    ) -> Self {
        let rng = StdRng::from_entropy();
        let len = data.len();
        let cnt_blocks = (len + blocksize - 1) / blocksize;
        let sol = Soliton::robust(cnt_blocks as i64, true, 0.1, 0.3);
        Encoder {
            data,
            len,
            blocksize,
            rng,
            dist: Uniform::new(0, cnt_blocks),
            cnt_blocks,
            sol,
            cnt: 0,
            encodertype,
        }
    }

    pub fn ideal(data: Vec<u8>, blocksize: usize, encodertype: EncoderType) -> Self {
        let rng = StdRng::from_entropy();
        let len = data.len();
        let cnt_blocks = (len + blocksize - 1) / blocksize;
        let sol = Soliton::ideal(cnt_blocks);
        Self {
            data,
            len,
            blocksize,
            rng,
            dist: Uniform::new(0, cnt_blocks),
            cnt_blocks,
            sol,
            cnt: 0,
            encodertype,
        }
    }

    pub fn drop(&mut self) -> Droplet {
        let mut r = vec![0; self.blocksize];

        let drop = match self.encodertype {
            EncoderType::Random => {
                let degree = self.sol.sample(&mut self.rng);
                let seed = self.rng.gen::<u64>();
                let sample = get_sample_from_rng_by_seed(seed, self.dist, degree);

                for k in sample {
                    let begin = k * self.blocksize;
                    let end = cmp::min((k + 1) * self.blocksize, self.len);
                    xor_bytes(&mut r, &self.data[begin..end]);
                }
                Droplet::new(DropType::Seeded(seed, degree), r)
            }
            EncoderType::Systematic => {
                let begin = (self.cnt % self.cnt_blocks) * self.blocksize;
                let end = cmp::min(
                    ((self.cnt % self.cnt_blocks) + 1) * self.blocksize,
                    self.len,
                );

                for (src_dat, drop_dat) in self.data[begin..end].iter().zip(r.iter_mut()) {
                    *drop_dat = *src_dat;
                }
                if (self.cnt + 2) > self.cnt_blocks * 2 {
                    self.encodertype = EncoderType::Random;
                }
                Droplet::new(DropType::Edges(self.cnt % self.cnt_blocks), r)
            }
            EncoderType::SysLdpc(code, _session) => {
                let begin = (self.cnt % self.cnt_blocks) * self.blocksize;
                let end = cmp::min(
                    ((self.cnt % self.cnt_blocks) + 1) * self.blocksize,
                    self.len,
                );

                for (src_dat, drop_dat) in self.data[begin..end].iter().zip(r.iter_mut()) {
                    *drop_dat = *src_dat;
                }
                if (self.cnt + 2) > self.cnt_blocks * 2 {
                    self.encodertype = EncoderType::RandLdpc(code, _session);
                }
                let mut drop = Droplet::new(DropType::Edges(self.cnt % self.cnt_blocks), r);
                droplet_encode(&mut drop, code);
                drop
            }
            EncoderType::RandLdpc(code, _session) => {
                let degree = self.sol.sample(&mut self.rng);
                let seed = self.rng.gen::<u64>();
                let sample = get_sample_from_rng_by_seed(seed, self.dist, degree);

                for k in sample {
                    let begin = k * self.blocksize;
                    let end = cmp::min((k + 1) * self.blocksize, self.len);
                    xor_bytes(&mut r, &self.data[begin..end]);
                }
                let mut drop = Droplet::new(DropType::Seeded(seed, degree), r);
                droplet_encode(&mut drop, code);
                drop

            }
        };

        self.cnt += 1;
        drop
    }
}

pub fn get_sample_from_rng_by_seed(
    seed: u64,
    range: rand::distributions::Uniform<usize>,
    degree: usize,
) -> impl Iterator<Item = usize> {
    let rng: StdRng = SeedableRng::seed_from_u64(seed as u64);
    rng.sample_iter(range).take(degree)
}

impl Iterator for Encoder {
    type Item = Droplet;
    fn next(&mut self) -> Option<Droplet> {
        Some(self.drop())
    }
}

#[derive(Clone, Debug)]
pub enum EncoderType {
    /// The first k symbols of a systematic Encoder correspond to the first k source symbols
    /// In case there is no loss, no repair needed. After the first k symbols are sent, it continous
    /// like in the Random case.
    Systematic,
    /// Begins immediately with random encoding.
    /// This may be a better choice when used with high-loss channels.
    Random,
    /// Systematic encoder, but wrapping droplets in LDPC codes of chosen byte size
    SysLdpc(LDPCCode, u32),
    /// Random encoding but wrapping droplets in LDPC codes of chosen byte size
    RandLdpc(LDPCCode, u32),
}
