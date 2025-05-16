mod VolumeStorage;
mod dataTypes;
mod MortonEncoding;
mod VolumeNetUtil;

use std::borrow::Borrow;
use serde::{Deserialize, Serialize};
use std::{io, thread};
use std::collections::HashMap;
use std::net::{TcpListener, TcpStream, Shutdown, SocketAddr, Ipv4Addr};
use std::io::{Read, Write};
use std::vec;
use std::str::from_utf8;
use std::sync::mpsc;
use std::sync::mpsc::{Sender, Receiver, TryRecvError};
use crate::dataTypes::{CHUNK_SIDE_LENGTH, CHUNK_SIZE, Point};
use crate::VolumeNetUtil::{AUTHORITY_ID, ChannelDataContainer, ChunkChangeRegistrations, NetChunk, NetChunkRequest, NetChunkRequestList, NetDeRegisterRequest, NetDiff, NetDiffList, NetPayload, ServerContext};
use crate::VolumeStorage::{Operations, Storage};

const HEADER_SIZE: usize = 5;

struct Config {
    pub server_ip: Ipv4Addr,
    pub port: u16,
}

fn handle_request(client_id: i32, has_authority: bool, request_label: &str, data: String, mut server_context: &mut ServerContext) {
    println!("Handling {} Request from client {}", request_label, client_id);
    match request_label {
        "diff" => {
            //Server only, perform one change, send out changes to server and clients
            if has_authority {
                let diff: NetDiff = serde_json::from_slice(&data.as_ref()).unwrap();
                server_context.volume_storage.set_relative(diff.x,
                                                           diff.y,
                                                           diff.z,
                                                           diff.chunk_id,
                                                           Point {
                                                            density: diff.density,
                                                            material: diff.material
                                                        });
                let client_update_list = server_context.change_registrations.registrations.get(&diff.chunk_id).unwrap().keys();

                for client in client_update_list {
                    server_context.client_send_channels.get(client).unwrap().send( ChannelDataContainer {
                        client_id,
                        payload: NetPayload {
                            payload_type: String::from(request_label.clone()),
                            data: data.clone()
                        }
                    });
                }
            }
        },
        "diffList" => {
            //Perform diff (above) for a list of diffs (still server only)
            if has_authority {
                let diff_list: NetDiffList = serde_json::from_slice(&data.as_ref()).unwrap();
                for diff in diff_list.list {
                    server_context.volume_storage.set_relative(diff.x,
                                                               diff.y,
                                                               diff.z,
                                                               diff.chunk_id,
                                                               Point {
                                                                density: diff.density,
                                                                material: diff.material
                                                            });
                    let client_update_list = server_context.change_registrations.registrations.get(&diff.chunk_id).unwrap().keys();
                    for client in client_update_list {
                        server_context.client_send_channels.get(client).unwrap().send( ChannelDataContainer {
                            client_id,
                            payload: NetPayload {
                                payload_type: String::from(request_label.clone()),
                                data: data.clone()
                            }
                        });
                    }
                }
            }
        },
        "chunkRequest" => {
            //Send chunk data to requesting client, and make sure server also has chunk.
            //Then register both client and server (if server is not already) for updates

            //If server is requesting a chunk, make a special registration that wont be de-registered
            //unless server requests to
            let chunk_req: NetChunkRequest = serde_json::from_slice(data.as_ref()).unwrap();
            let chunk_data = NetChunk::from_chunk(server_context.volume_storage.get_chunk(chunk_req.x, chunk_req.y, chunk_req.z),
                                                  chunk_req.x, chunk_req.y, chunk_req.z);
            let chunk_id = server_context.volume_storage.get_chunk_id(chunk_req.x, chunk_req.y, chunk_req.z);

            server_context.change_registrations.register_for_chunk_changes(has_authority, client_id, chunk_id);
            server_context.client_send_channels.get(&client_id).unwrap().send( ChannelDataContainer {
                client_id,
                payload: NetPayload {
                    payload_type: String::from(request_label.clone()),
                    data: serde_json::to_string(&chunk_data).unwrap()
                }
            });
        }
        "chunkRequestList" => {
            //Perform chunkRequest for a list of chunks
            let chunk_req_list: NetChunkRequestList = serde_json::from_slice(data.as_ref()).unwrap();
            for chunkReq in chunk_req_list.list {
                let chunk_data = NetChunk::from_chunk(server_context.volume_storage.get_chunk(chunkReq.x, chunkReq.y, chunkReq.z),
                                                      chunkReq.x, chunkReq.y, chunkReq.z);
                let chunk_id = server_context.volume_storage.get_chunk_id(chunkReq.x, chunkReq.y, chunkReq.z);

                server_context.change_registrations.register_for_chunk_changes(has_authority, client_id, chunk_id);
                server_context.client_send_channels.get(&client_id).unwrap().send( ChannelDataContainer {
                    client_id,
                    payload: NetPayload {
                        payload_type: String::from(request_label.clone()),
                        data: serde_json::to_string(&chunk_data).unwrap()
                    }
                });
            }
        }
        "unregisterChunk" => {
            //unregister a client from receiving updates for a chunk.
            let Deregister_req: NetDeRegisterRequest = serde_json::from_slice(data.as_ref()).unwrap();
            let chunk_id = server_context.volume_storage.get_chunk_id(Deregister_req.x, Deregister_req.y, Deregister_req.z);
            server_context.change_registrations.deregister_for_chunk_changes(client_id, chunk_id);
        }
        "claimAuthority" => {
            //client can login to being server authority (should be used by UE server authority)
        }
        &_ => {}
    }
}

fn client_loop(mut stream: TcpStream, client_id: i32, send_channel: Sender<ChannelDataContainer>, receive_channel: Receiver<ChannelDataContainer>) {
    loop {
        //Take in request
        let mut header = [0 as u8; 4]; // get data size
        match stream.peek(&mut header) {
            Ok(size) => {
                if size != 0 {
                    match stream.read_exact(&mut header) {
                        Ok(size) => {
                            let data_size: u32 = u32::from_be_bytes(header);
                            println!("Received Request from client {} of size: {}b",  client_id, data_size);
                            let mut buffer = vec![0 as u8; data_size as usize];
                            match stream.read_exact(&mut buffer) {
                                Ok(size) => {
                                    let received_payload: NetPayload = serde_json::from_slice(&buffer).unwrap();
                                    send_channel.send(
                                        ChannelDataContainer {
                                            client_id: client_id as i32,
                                            payload: received_payload
                                        }
                                    );
                                },
                                Err(_) => {}
                            }

                        },
                        Err(_) => {
                            println!("An error occurred, terminating connection with {}", stream.peer_addr().unwrap());
                            stream.shutdown(Shutdown::Both).unwrap();
                        }
                    }
                }
            }
            Err(e) => {}
        }

        //Send Data to clients
        match receive_channel.try_recv() {
            Ok(channel_data_container) => {
                let serialized_payload = serde_json::to_string(&channel_data_container.payload).unwrap();
                let mut header = serialized_payload.len() as u32;

                stream.write(&header.to_be_bytes());
                stream.write(serialized_payload.as_ref()).unwrap();
            }
            Err(_) => {}
        }
    }

}

fn read_config() -> Config{

    return Config {
        server_ip: Ipv4Addr::new(0, 0, 0, 0),
        port: 6969
    }
}

fn main() {
    //Get Config and apply settings
    println!("Initializing Config");
    let config = read_config();

    let ip = config.server_ip.clone();
    let port= config.port.clone();

    //Initialize ServerContext
    println!("Initializing Server Context");
    let mut server_context = ServerContext {
        change_registrations: ChunkChangeRegistrations::new(),
        volume_storage: VolumeStorage::Storage::new(),
        client_send_channels: HashMap::new()
    };

    //Generate Terrain
    println!("Generating Level");
    for x in 0..(CHUNK_SIDE_LENGTH*2) {
        for y in 0..(CHUNK_SIDE_LENGTH*2) {
            for z in 0..(CHUNK_SIDE_LENGTH*2) {
                if z < 10 {
                    server_context.volume_storage.set_global(x, y, z, Point {
                        density: 100,
                        material: 1
                    })
                }
            }
        }
    }

    //Open server for connections
    let addr = SocketAddr::from((ip, port));
    let listener = TcpListener::bind(addr).unwrap();
    listener.set_nonblocking(true).expect("Cannot set non-blocking");
    println!("Server listening on port {}" , port);

    //Create channel for thread to send data to main thread
    let (tx, rx) = mpsc::channel();

    let mut client_connection_id = 0;
    loop {
        //Check and open new threads per connection
        match listener.accept() {
            Ok((stream, socketAddr)) => {
                println!("New connection: {}", socketAddr);

                //Make client specific channels
                let tx_new = tx.clone();
                let (tx_client, rx_client) = mpsc::channel();
                server_context.client_send_channels.insert(client_connection_id.clone(), tx_client);

                //Start client loop
                thread::spawn(move|| {
                    client_loop(stream, client_connection_id, tx_new, rx_client)
                });

                //increment id for next client to join;
                client_connection_id = client_connection_id + 1;
            }
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
            }
            Err( e) => {
                println!("Error: {}", e);
            }
        }

        //Perform request
        match rx.try_recv() {
            Ok(channel_data_container) => {
                handle_request(channel_data_container.client_id,
                               channel_data_container.client_id == AUTHORITY_ID,
                               &channel_data_container.payload.payload_type,
                               channel_data_container.payload.data,
                               &mut server_context)
            }
            Err(_) => {}
        }
    }
}
