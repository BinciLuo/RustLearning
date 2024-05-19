mod cache;
mod dns;
mod utils;
mod tests;
use std::sync::Arc;

use log::{self, LogLevel};

use crate::dns::DNSServer;

fn main() {
    let dns_server = Arc::new(DNSServer::new("Local DNS", LogLevel::Debug));
    DNSServer::run_processing_command(&dns_server);
    // DNSServer::run_processing_request(&dns_server);
    DNSServer::wait_exit(&dns_server);
}
