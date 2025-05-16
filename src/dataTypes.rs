use std::vec::Vec;
use serde::{Deserialize, Serialize};

pub const CHUNK_SIDE_LENGTH: u32 = 32;
pub const CHUNK_SIZE: usize = usize::pow(32, 3);

//Storage Structs
pub struct Point {
    pub(crate) density: u8,
    pub(crate) material: u8
}

pub struct Voxel {
    density: [u8; 8],
    material: [u8; 8]
}

pub struct Chunk {
    pub(crate) density: Vec<u8>,
    pub(crate) material: Vec<u8>
}

