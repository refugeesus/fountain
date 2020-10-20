use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum DropType {
    /// First is seed, second degree
    Seeded(u64, usize),
    /// Just a list of edges
    Edges(usize),
}

/// A Droplet is created by the Encoder.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Droplet {
    /// The droptype can be based on seed or a list of edges
    pub droptype: DropType,
    /// The payload of the Droplet
    pub data: Vec<u8>,
}

impl Droplet {
    pub fn new(droptype: DropType, data: Vec<u8>) -> Droplet {
        Droplet { droptype, data }
    }
}

#[derive(Debug, Clone)]
pub struct RxDroplet {
    pub edges_idx: Vec<usize>,
    pub data: Vec<u8>,
}
