use env_logger;
use grin_core::core::hash::Hashed;
use grin_core::pow::Difficulty;
use grin_p2p::handshake::Handshake;
use std::net::{IpAddr, Shutdown, SocketAddr, TcpListener, TcpStream};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use grin_core::core;
use grin_core::core::hash::Hash;
use grin_core::genesis::genesis_floo;
use grin_core::global::{set_mining_mode, ChainTypes};
use grin_p2p::types::{
    Capabilities, ChainAdapter, Error, NetAdapter, P2PConfig, PeerAddr, PeerInfo, ReasonForBan,
    TxHashSetRead,
};
use grin_p2p::Peer;
use grin_p2p::Server;
use log::*;

mod adapter;
use crate::adapter::FakeAdapter;
use grin_util::StopState;

fn main() {
    env_logger::init();
    info!("Hello, world!");
    set_mining_mode(ChainTypes::Floonet);
    let mut cfg = P2PConfig::default();
    cfg.host = "0.0.0.0".parse().unwrap();
    cfg.port = 13415;

    let handshake = Arc::new(Handshake::new(genesis_floo().hash(), cfg.clone()));

    thread::spawn({
        let handshake = handshake.clone();
        let local_addr = PeerAddr(SocketAddr::new(cfg.host, cfg.port));
        move || {
            let peer_addr = PeerAddr(SocketAddr::new("127.0.0.1".parse().unwrap(), 13414));
            let res = connect(peer_addr, local_addr, &handshake);
            println!("\x1B[35;1m res\x1B[0m = {:?}", res);
        }
    });
    thread::sleep(Duration::from_secs(30));
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
