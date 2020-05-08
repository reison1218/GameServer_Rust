mod entity;
mod mgr;
mod net;
use std::sync::atomic::AtomicU32;
use std::sync::{Arc, Mutex, RwLock};

#[macro_use]
extern crate lazy_static;

lazy_static! {
    static ref GATE_ID:Arc<RwLock<AtomicU32>> ={
        let mut arc: Arc<RwLock<AtomicU32>> = Arc::new(RwLock::new(AtomicU32::new(0)));
        arc
    };

    static ref ROOM_ID:Arc<RwLock<AtomicU32>> ={
        let mut arc: Arc<RwLock<AtomicU32>> = Arc::new(RwLock::new(AtomicU32::new(0)));
        arc
    };
}

fn main() {
    println!("Hello, world!");
}
