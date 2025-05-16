use std::collections::HashMap;
use crate::dataTypes;
use crate::MortonEncoding;
use crate::dataTypes::{Chunk, CHUNK_SIDE_LENGTH, CHUNK_SIZE, Point};

const GRID_SIDE_LENGTH: u32 = 32;

pub struct Storage {
    chunks : HashMap<u32, Chunk>
}

pub trait Operations {
    fn get(&self, x: u32, y: u32, z: u32) -> Point;

    fn set_global(&mut self, x: u32, y: u32, z: u32, point: Point);

    fn set_relative(&mut self, x: u32, y: u32, z: u32, chunk_id: u32, point: Point);

    fn get_chunk(&self, x: u32, y: u32, z: u32) -> Chunk;

    fn create_chunk(&mut self, x: u32, y: u32, z: u32);

    fn add_chunk(&mut self, x: u32, y: u32, z: u32, chunk: Chunk);

    fn get_chunk_id(&self, x: u32, y: u32, z: u32) -> u32;
}

impl Storage {
    pub fn new() -> Storage {
        return Storage {
            chunks: HashMap::new()
        };
    }

    pub fn getChunkCount(&self) -> usize {
        return self.chunks.len();
    }

    pub fn listChunkCoords(&self) {
        for chunk in &self.chunks {
            println!("chunk ID: {}", chunk.0);
        }
    }

}

impl Operations for Storage {
     fn get(&self, x: u32, y: u32, z: u32) -> Point {
        //Get correct chunk
        let x_chunk = x / CHUNK_SIDE_LENGTH;
        let y_chunk = y / CHUNK_SIDE_LENGTH;
        let z_chunk = z / CHUNK_SIDE_LENGTH;

        //Get coordinates relative to chunk
        let rel_x = x % CHUNK_SIDE_LENGTH;
        let rel_y = y % CHUNK_SIDE_LENGTH;
        let rel_z = z % CHUNK_SIDE_LENGTH;

        let key = self.get_chunk_id(x_chunk, y_chunk, z_chunk);
        let mut chunk = self.chunks.get(&key).unwrap();
        let i: usize = MortonEncoding::morton_encode(rel_x, rel_y, rel_z, CHUNK_SIDE_LENGTH) as usize;
        return Point{
            density: *chunk.density.get(i).unwrap(),
            material: *chunk.material.get(i).unwrap()
        };

    }

    fn set_global(&mut self, x: u32, y: u32, z: u32, point: Point) {
        //Get correct chunk
        let x_chunk = x / CHUNK_SIDE_LENGTH;
        let y_chunk = y / CHUNK_SIDE_LENGTH;
        let z_chunk = z / CHUNK_SIDE_LENGTH;

        if x_chunk < GRID_SIDE_LENGTH && y_chunk < GRID_SIDE_LENGTH && z_chunk < GRID_SIDE_LENGTH {
            //Get coordinates relative to chunk
            let rel_x = x % CHUNK_SIDE_LENGTH;
            let rel_y = y % CHUNK_SIDE_LENGTH;
            let rel_z = z % CHUNK_SIDE_LENGTH;

            let key = self.get_chunk_id(x_chunk, y_chunk, z_chunk);
            let i: usize = MortonEncoding::morton_encode(rel_x, rel_y, rel_z, CHUNK_SIDE_LENGTH) as usize;

            if !self.chunks.contains_key(&key) {
                self.create_chunk(x_chunk, y_chunk, z_chunk);
            }
            self.chunks.get_mut(&key).unwrap().density[i] = point.density.clone();
            self.chunks.get_mut(&key).unwrap().material[i] = point.material.clone();
        }
    }

    fn set_relative(&mut self, x: u32, y: u32, z: u32, chunk_id: u32, point: Point) {
        let i: usize = MortonEncoding::morton_encode(x, y, z, CHUNK_SIDE_LENGTH) as usize;
        if self.chunks.contains_key(&chunk_id) {
            self.chunks.get_mut(&chunk_id).unwrap().density[i] = point.density.clone();
            self.chunks.get_mut(&chunk_id).unwrap().material[i] = point.material.clone();
        }
    }

    fn get_chunk(&self, x: u32, y: u32, z: u32) -> Chunk {
        let key = x + GRID_SIDE_LENGTH*y + GRID_SIDE_LENGTH*GRID_SIDE_LENGTH*z;
        return if self.chunks.contains_key(&key) {
            let temp = self.chunks.get(&key).unwrap();
            Chunk {
                density: temp.density.clone(),
                material: temp.material.clone()
            }
        } else {
            Chunk {
                density: vec![0; CHUNK_SIZE],
                material: vec![0; CHUNK_SIZE],
            }
        }
    }

    fn create_chunk(&mut self, x: u32, y: u32, z: u32) {
        let key = self.get_chunk_id(x, y, z);
        self.chunks.insert(key, Chunk{
            density: vec![0; CHUNK_SIZE],
            material: vec![0; CHUNK_SIZE]
        });
    }

    fn add_chunk(&mut self, x: u32, y: u32, z: u32, chunk: Chunk) {
        let key = self.get_chunk_id(x, y, z);
        self.chunks.insert(key, Chunk{
            density: chunk.density.clone(),
            material: chunk.material.clone()
        });
    }

    fn get_chunk_id(&self, x: u32, y: u32, z: u32) -> u32 {
        return x + GRID_SIDE_LENGTH*y + GRID_SIDE_LENGTH*GRID_SIDE_LENGTH*z;
    }
}


