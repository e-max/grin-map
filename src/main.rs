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

type Storage = HashMap<PeerAddr, Option<Vec<PeerAddr>>>;

const NTHREADS: u8 = 100;

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
    let local_addr = PeerAddr(SocketAddr::new(cfg.host, cfg.port));

    let mut count = 1;

    queue.push(peer_addr);

    //   let mut threads = vec![];

    let t = worker(
        handshake.clone(),
        local_addr.clone(),
        queue.clone(),
        storage.clone(),
        count,
    );
    //    threads.push(t);

    loop {
        if Arc::strong_count(&queue) == 1 {
            break;
        }
        if queue.len() > 5 {
            info!(
                "Too many items in queue. Start new thread (was {})",
                Arc::strong_count(&queue)
            );
            if Arc::strong_count(&queue) < 100 {
                count += 1;
                worker(
                    handshake.clone(),
                    local_addr.clone(),
                    queue.clone(),
                    storage.clone(),
                    count,
                );
            } else {
                info!("reached thread limits");
            }
        }
        thread::sleep(Duration::from_secs(1));
    }

    let mut public = 0;
    for (k, v) in &(*storage.read().unwrap()) {
        println!("\x1B[35;1m v\x1B[0m = {:?}", v);
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

fn worker(
    handshake: Arc<Handshake>,
    local_addr: PeerAddr,
    queue: Arc<SegQueue<PeerAddr>>,
    storage: Arc<ShardedLock<Storage>>,
    i: u64,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        info!("start thread {}", i);
        loop {
            info!("New iterations");
            let peer_addr = match queue.pop() {
                Ok(addr) => addr,
                Err(e) => {
                    warn!("Queue is emtpy. Quit thread {}.", i);
                    break;
                }
            };
            if storage.write().unwrap().contains_key(&peer_addr) {
                continue;
            }
            info!("Thread {} got add {}", i, peer_addr);
            match connect(peer_addr, local_addr, &handshake) {
                Ok(addrs) => {
                    for a in &addrs {
                        queue.push(a.clone());
                    }
                    let mut hm = storage.write().unwrap();
                    hm.insert(peer_addr, Some(addrs));
                }
                Err(e) => {
                    debug!("Cannot connect: {}", e);
                    let mut hm = storage.write().unwrap();
                    hm.insert(peer_addr, None);
                }
            }
            //info!("Now sleep for 1 sec");
            //thread::sleep(Duration::from_secs(1));
            info!("end of loop");
        }
    })
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

    peer.send_peer_request(Capabilities::PEER_LIST)
        .map_err(|e| format!("{:?}", e))?;

    adapter.get_peers().map_err(|e| format!("{:?}", e))
}
