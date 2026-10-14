use std::{
    sync::{
        Arc,
        Mutex,
    },
    thread
};

use super::muxer::Muxer;

/// Spawns a new thread for the muxer to handle writing packets to the output file.
///
/// Arguments
/// * `muxer` - An Arc<Mutex<Muxer>> for thread-safe access to the muxer.
pub fn spawn(
    muxer: Arc<Mutex<Muxer>>,
) -> thread::JoinHandle<()>{
    let muxer_thread = thread::Builder::new()
        .name("muxer".to_string())
        .spawn(move || {
            let mut muxer = muxer.lock().unwrap();
            muxer.initialize().unwrap();
            muxer.write_packets().unwrap();
        }).unwrap();

    muxer_thread
}