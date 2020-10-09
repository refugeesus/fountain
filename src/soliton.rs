use rand::{distributions::Distribution, Rng};
use statrs::distribution::RobustSoliton;
#[derive(Debug, Clone)]
pub enum Soliton {
    Ideal {
        limit: f32,
    },
    Robust {
        sol: RobustSoliton,
    },
}

impl Soliton {
    pub fn ideal(k: usize) -> Self {
        Self::Ideal {
            limit: 1.0 / (k as f32),
        }
    }
    pub fn robust(blocks: i64, heuristic: bool, ripple: f64, fail_probability: f64) -> Self {
        Self::Robust{
            sol: RobustSoliton::new(blocks, heuristic, ripple, fail_probability).unwrap()
        }
    }
}

impl Distribution<usize> for Soliton {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> usize {
        match self {
            Self::Ideal { limit } => {
                let y = rng.gen::<f32>();
                if y >= *limit {
                    (1.0 / y).ceil() as usize
                } else {
                    1
                }
            }
            Self::Robust { sol } => {
                sol.query_table(rng).unwrap() as usize
            }
        }
    }
}
