use serde::{Serialize, Deserialize};
use bincode;

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

pub enum BincodeError {
    DecodeError,
    EncodeError,
}

pub fn from_binary(data: Vec<u8>) -> Result<Droplet, BincodeError> {
    let drop: Option<Droplet> = bincode::deserialize(data.as_ref()).unwrap_or(None);
    match drop {
        Some(d) => Ok(d),
        None => Err(BincodeError::DecodeError),
    }
}

pub fn to_binary(droplet: Droplet) -> Result<Vec<u8>, BincodeError> {
    let data: Option<Vec<u8>> = bincode::serialize(&droplet).ok();
    match data {
        Some(d) => Ok(d),
        None => Err(BincodeError::EncodeError),
    }
}
