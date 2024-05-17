mod cache;
mod dns;
mod utils;
mod tests;
use std::{
    sync::{Arc, Mutex},
    thread::JoinHandle,
};

use log::{self, LogLevel};

use crate::dns::DNSServer;

fn main() {
    let dns_server = Arc::new(Mutex::new(DNSServer::new("Local DNS", LogLevel::Debug)));
    let mut handles = Vec::<JoinHandle<()>>::new();
    DNSServer::run_listening(&dns_server, &mut handles);
    DNSServer::run_control(&dns_server, &mut handles);
    for handle in handles {
        handle.join().unwrap();
    }
}
