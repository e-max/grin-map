use env_logger;
use grin_core::core::hash::Hashed;
use grin_core::pow::Difficulty;
use grin_p2p::handshake::Handshake;
use std::net::{SocketAddr, TcpStream};
use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use std::time::Duration;

use grin_core::genesis::genesis_floo;
use grin_core::global::{set_mining_mode, ChainTypes};
use grin_p2p::types::{Capabilities, P2PConfig, PeerAddr};
use grin_p2p::Peer;
use log::*;
use std::collections::{HashMap, VecDeque};

mod adapter;
use crate::adapter::FakeAdapter;
use crossbeam_queue::SegQueue;
use crossbeam_utils::sync::ShardedLock;

fn main() {
    env_logger::init();
    info!("Hello, world!");
    set_mining_mode(ChainTypes::Floonet);
    let mut cfg = P2PConfig::default();
    cfg.host = "0.0.0.0".parse().unwrap();
    cfg.port = 13415;

    let handshake = Arc::new(Handshake::new(genesis_floo().hash(), cfg.clone()));

    let queue = Arc::new(SegQueue::new());
    let hm: HashMap<PeerAddr, Option<Vec<PeerAddr>>> = HashMap::new();
    let storage = Arc::new(ShardedLock::new(hm));
    //let peer_addr = PeerAddr(SocketAddr::new("127.0.0.1".parse().unwrap(), 13414));
    let peer_addr = PeerAddr(SocketAddr::new("35.157.247.209".parse().unwrap(), 13414));

    queue.push(peer_addr);

    let t = thread::spawn({
        let handshake = handshake.clone();
        let local_addr = PeerAddr(SocketAddr::new(cfg.host, cfg.port));
        let queue = queue.clone();
        let storage = storage.clone();
        move || loop {
            info!("New iterations");
            let peer_addr = match queue.pop() {
                Ok(addr) => addr,
                Err(e) => {
                    warn!("Queue is emtpy. Quit.");
                    break;
                }
            };
            if storage.write().unwrap().contains_key(&peer_addr) {
                continue;
            }
            println!("\x1B[33;1m peer_addr\x1B[0m = {:?}", peer_addr);
            match connect(peer_addr, local_addr, &handshake) {
                Ok(addrs) => {
                    println!("\x1B[32;1m addrs\x1B[0m = {:?}", addrs);
                    for a in &addrs {
                        queue.push(a.clone());
                    }
                    let mut hm = storage.write().unwrap();
                    hm.insert(peer_addr, Some(addrs));
                }
                Err(e) => {
                    println!("\x1B[31;1m e\x1B[0m = {:?}", e);
                    let mut hm = storage.write().unwrap();
                    hm.insert(peer_addr, None);
                }
            }
            //info!("Now sleep for 1 sec");
            //thread::sleep(Duration::from_secs(1));
        }
    });
    t.join();

    let mut public = 0;
    for (k, v) in &(*storage.read().unwrap()) {
        if v.is_some() {
            public += 1;
        }
    }
    println!(
        "Result: total {}, public: {}",
        storage.read().unwrap().len(),
        public
    );
}

fn connect(
    peer_addr: PeerAddr,
    local_addr: PeerAddr,
    handshake: &Handshake,
) -> Result<Vec<PeerAddr>, String> {
    let stream = TcpStream::connect_timeout(&peer_addr.0, Duration::from_secs(10))
        .map_err(|e| format!("{}", e))?;

    let adapter = Arc::new(FakeAdapter::new());

    let peer = Peer::connect(
        stream,
        Capabilities::PEER_LIST,
        Difficulty::min(),
        local_addr,
        handshake,
        adapter.clone(),
    )
    .map_err(|e| format!("{:?}", e))?;

    let peer = Arc::new(peer);

    println!("\x1B[31;1m peer\x1B[0m = {:?}", peer);
    peer.send_peer_request(Capabilities::PEER_LIST)
        .map_err(|e| format!("{:?}", e))?;

    adapter.get_peers().map_err(|e| format!("{:?}", e))
}
