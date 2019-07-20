use chrono::prelude::{DateTime, Utc};
use grin_chain as chain;
use grin_core::core;
use grin_core::core::hash::Hash;
use grin_core::pow::Difficulty;
use grin_p2p::types::{
    Capabilities, ChainAdapter, Error, NetAdapter, P2PConfig, PeerAddr, PeerInfo, ReasonForBan,
    TxHashSetRead,
};
use log::*;
use std::fs::File;
use std::io::{self, Read};
use std::path::PathBuf;
use std::time::Duration;

use crossbeam_channel::RecvTimeoutError;
use crossbeam_channel::{bounded, Receiver, Sender};

pub struct FakeAdapter {
    sender: Sender<Vec<PeerAddr>>,
    receiver: Receiver<Vec<PeerAddr>>,
}

impl FakeAdapter {
    pub fn new() -> Self {
        let (sender, receiver) = bounded(1);
        FakeAdapter { sender, receiver }
    }
    pub fn get_peers(&self) -> Result<Vec<PeerAddr>, RecvTimeoutError> {
        self.receiver.recv_timeout(Duration::from_secs(2))
    }
}

impl ChainAdapter for FakeAdapter {
    fn total_difficulty(&self) -> Result<Difficulty, chain::Error> {
        Ok(Difficulty::min())
    }
    fn total_height(&self) -> Result<u64, chain::Error> {
        Ok(0)
    }
    fn get_transaction(&self, _h: Hash) -> Option<core::Transaction> {
        None
    }

    fn tx_kernel_received(&self, _h: Hash, _peer_info: &PeerInfo) -> Result<bool, chain::Error> {
        Ok(true)
    }
    fn transaction_received(
        &self,
        _: core::Transaction,
        _stem: bool,
    ) -> Result<bool, chain::Error> {
        Ok(true)
    }
    fn compact_block_received(
        &self,
        _cb: core::CompactBlock,
        _peer_info: &PeerInfo,
    ) -> Result<bool, chain::Error> {
        Ok(true)
    }
    fn header_received(
        &self,
        _bh: core::BlockHeader,
        _peer_info: &PeerInfo,
    ) -> Result<bool, chain::Error> {
        Ok(true)
    }
    fn block_received(&self, _: core::Block, _: &PeerInfo, _: bool) -> Result<bool, chain::Error> {
        Ok(true)
    }
    fn headers_received(
        &self,
        _: &[core::BlockHeader],
        _: &PeerInfo,
    ) -> Result<bool, chain::Error> {
        Ok(true)
    }
    fn locate_headers(&self, _: &[Hash]) -> Result<Vec<core::BlockHeader>, chain::Error> {
        Ok(vec![])
    }
    fn get_block(&self, _: Hash) -> Option<core::Block> {
        None
    }
    fn kernel_data_read(&self) -> Result<File, chain::Error> {
        unimplemented!()
    }
    fn kernel_data_write(&self, _reader: &mut Read) -> Result<bool, chain::Error> {
        unimplemented!()
    }
    fn txhashset_read(&self, _h: Hash) -> Option<TxHashSetRead> {
        unimplemented!()
    }

    fn txhashset_receive_ready(&self) -> bool {
        false
    }

    fn txhashset_write(
        &self,
        _h: Hash,
        _txhashset_data: File,
        _peer_info: &PeerInfo,
    ) -> Result<bool, chain::Error> {
        Ok(false)
    }

    fn txhashset_download_update(
        &self,
        _start_time: DateTime<Utc>,
        _downloaded_size: u64,
        _total_size: u64,
    ) -> bool {
        false
    }

    fn get_tmp_dir(&self) -> PathBuf {
        unimplemented!()
    }

    fn get_tmpfile_pathname(&self, _tmpfile_name: String) -> PathBuf {
        unimplemented!()
    }
}

impl NetAdapter for FakeAdapter {
    fn find_peer_addrs(&self, _: Capabilities) -> Vec<PeerAddr> {
        vec![]
    }
    fn peer_addrs_received(&self, peers: Vec<PeerAddr>) {
        println!("\x1B[31;1m peers\x1B[0m = {:?}", peers);
        info!("PEERS!");
        self.sender.send(peers).unwrap();
    }
    fn peer_difficulty(&self, _: PeerAddr, _: Difficulty, _: u64) {}
    fn is_banned(&self, _: PeerAddr) -> bool {
        false
    }
}
