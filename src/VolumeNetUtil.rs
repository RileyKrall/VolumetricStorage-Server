use std::collections::HashMap;
use std::sync::mpsc::Sender;
use std::vec::Vec;
use serde::{Deserialize, Serialize};
use crate::dataTypes::Chunk;
use crate::VolumeStorage::Storage;

pub const AUTHORITY_ID: i32 = 0;

pub struct ServerContext {
    pub change_registrations: ChunkChangeRegistrations,
    pub volume_storage: Storage,
    pub client_send_channels: HashMap<i32, Sender<ChannelDataContainer>>
}

pub struct ChannelDataContainer {
    pub(crate) client_id: i32,
    pub(crate) payload: NetPayload
}

#[derive(Deserialize, Serialize)]
pub struct NetPayload {
    pub(crate) payload_type: String,
    pub(crate) data: String
}

#[derive(Deserialize, Serialize)]
pub struct NetDiffList {
    pub(crate) list: Vec<NetDiff>
}

#[derive(Deserialize, Serialize)]
pub struct NetDiff {
    pub chunk_id: u32,
    pub x: u32,
    pub y: u32,
    pub z: u32,
    pub density: u8,
    pub material: u8
}

#[derive(Deserialize, Serialize)]
pub struct NetChunkRequestList {
    pub(crate) list: Vec<NetChunkRequest>
}

#[derive(Deserialize, Serialize)]
pub struct NetChunkList {
    pub(crate) list: Vec<NetChunk>
}

#[derive(Deserialize, Serialize)]
pub struct NetChunk {
    pub x: u32,
    pub y: u32,
    pub z: u32,
    pub(crate) density: Vec<u8>,
    pub(crate) material: Vec<u8>
}

impl NetChunk {
    pub fn from_chunk(chunk: Chunk, x : u32, y : u32, z : u32) -> NetChunk {
        return NetChunk {
            x,
            y,
            z,
            density: chunk.density.clone(),
            material: chunk.material.clone()
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct NetChunkRequest {
    pub x: u32,
    pub y: u32,
    pub z: u32,
}

#[derive(Deserialize, Serialize)]
pub struct NetDeRegisterRequest {
    pub x: u32,
    pub y: u32,
    pub z: u32,
}

pub struct ChunkChangeRegistrations {
    pub registrations: HashMap<u32, HashMap<i32, bool>>,
}

impl ChunkChangeRegistrations {
    pub fn new() -> ChunkChangeRegistrations {
         return ChunkChangeRegistrations {
             registrations: HashMap::new()
         };
    }

    pub fn register_for_chunk_changes(&mut self, is_authority: bool, client_id: i32, chunk_id: u32) {
        if !self.registrations.contains_key(&chunk_id) {
            self.registrations.insert(chunk_id, HashMap::new());
        }
        self.registrations.get_mut(&chunk_id).unwrap().insert(client_id, is_authority);
    }

    pub fn deregister_for_chunk_changes(&mut self, client_id: i32, chunk_id: u32) {
        if !self.registrations.contains_key(&chunk_id) {
            self.registrations.get_mut(&chunk_id).unwrap().remove(&client_id);
        }
    }

    pub fn list_registrations(&self) {
        for chunk in &self.registrations {
            println!("Chunk: {}", chunk.0);
            for client in chunk.1 {
                println!("    Client: {}", client.0);
            }
        }
    }
}