use core::fmt;
use std::{
    collections::HashMap,
    fs::File,
    io::{self, Read, Write},
    net::UdpSocket,
    sync::{atomic::AtomicBool, mpsc::TryRecvError, Arc, Mutex},
    thread::{self, sleep, JoinHandle},
    time::Duration,
};

use log::{LogLevel, Logger};
use serde::{Deserialize, Serialize};

use crate::{
    cache::Cache,
    utils::{clean_io, exec_shell_command, query_google_dns},
};

#[derive(Debug, Deserialize)]
struct DNSRecord {
    domain: String,
    ip: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct Config {
    dns_config: String,
    dns_port: u16,
    remote_dns_addr: String,
    name: String,
    cache_time: u32,
}

// pub struct DNSServerArcMutex(Arc<Mutex<DNSServer>>);
// impl DNSServerArcMutex{
//     pub fn listening(){}
// }

pub struct DNSServer {
    dns_config: HashMap<String, String>,
    server_socket: UdpSocket,
    port: u16,
    name: String,
    remote_addr: String,
    config_file_path: String,
    logger: Logger,
    cache: Cache,
    stop_request: AtomicBool,
    exit: AtomicBool,
    handles: Vec<JoinHandle<()>>,
}

impl DNSServer {
    fn load_config_file(file_path: &str) -> io::Result<String> {
        let mut file = File::open(file_path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        Ok(contents)
    }

    fn parse_dns_config(contents: &str) -> Result<HashMap<String, String>, serde_json::Error> {
        let data: Vec<DNSRecord> = serde_json::from_str(contents)?;
        let dns_config = data
            .into_iter()
            .map(|record| (record.domain, record.ip))
            .collect();
        Ok(dns_config)
    }

    pub fn new(config_fp: &str, loglevel: LogLevel) -> Self {
        // Read config
        let mut file = File::open("config.json").expect("Unable to open file");
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .expect("Unable to read file");
        let config: Config = serde_json::from_str(&contents).expect("Unable to parse JSON");

        // Set Logger
        let logger = Logger::new(config.name.clone().as_str(), loglevel);
        logger.log(LogLevel::Warning, format!("Logger of DNS server initialized. {}", &logger));

        // Set Cache
        logger.log(LogLevel::Warning, "Setting Cache.");
        let cache = Cache::new(config.cache_time.into());
        logger.log(LogLevel::Debug, format!("{}", cache));
        logger.log(LogLevel::Warning, "Finish Setting Cache");

        // Create Socket
        logger.log(LogLevel::Warning, "Creating Socket");
        let server_socket =
            UdpSocket::bind(("0.0.0.0", config.dns_port)).expect("Failed to bind socket");
        let _ = server_socket.set_nonblocking(true);
        logger.log(LogLevel::Warning, "Finish Creating Socket");

        // Load DNS Config
        let contents =
            Self::load_config_file(&config.dns_config).expect("Failed to open dns config file");
        let dns_config = Self::parse_dns_config(&contents).expect("Failed to parse DNS config");

        DNSServer {
            dns_config,
            server_socket,
            port: config.dns_port,
            name: config.name,
            remote_addr: config.remote_dns_addr,
            config_file_path: config_fp.to_string(),
            logger,
            cache,
            stop_request: AtomicBool::new(false),
            exit: AtomicBool::new(false),
            handles: Vec::<JoinHandle<()>>::new(),
        }
    }

    pub fn add_handle(&mut self, handle: JoinHandle<()>) {
        self.handles.push(handle)
    }

    fn resolve_dns(&mut self, domain: &str) -> String {
        // Clean domain
        let cleaned_domain = clean_io(domain);

        // Search in cache
        if let Some(ip) = self.cache.get(&cleaned_domain, false) {
            self.logger
                .log(LogLevel::Info, &format!("Cached {}---->{}", domain, ip));
            return ip.clone();
        }

        // Search in local DNS
        if let Some(ip) = self.dns_config.get(cleaned_domain.as_str()) {
            self.logger
                .log(LogLevel::Info, &format!("Local DNS {}---->{}", domain, ip));
            self.cache.put(cleaned_domain.to_string(), ip.to_string());
            return ip.clone();
        }

        // Search Google DNS
        match query_google_dns(&cleaned_domain, "a") {
            Ok(dns_response) => {
                self.logger.log(
                    LogLevel::Warning,
                    &format!(
                        "Google DNS {}---->{}",
                        cleaned_domain.to_string(),
                        dns_response.ip_address
                    ),
                );
                self.cache
                    .put(cleaned_domain.to_string(), dns_response.ip_address.clone());
                return dns_response.ip_address;
            }
            Err(_) => {
                self.logger.log(
                    LogLevel::Error,
                    format!(
                        "Error occured when queryiny google dns, domian: {}",
                        cleaned_domain
                    ),
                );
            }
        }

        // Search system DNS
        let cmd = format!("dig +short {}", cleaned_domain);
        match exec_shell_command(&cmd) {
            Ok(ip) => {
                self.logger.log(
                    LogLevel::Warning,
                    &format!("Local DNS not found, system result: {}", ip),
                );
                self.cache.put(cleaned_domain.to_string(), ip.clone());
                ip
            }
            Err(e) => {
                self.logger.log(
                    LogLevel::Error,
                    &format!("Exception occurred: {}\n Command: {}", e, cmd),
                );
                String::new()
            }
        }
    }

    pub fn _listening(&mut self) {
        self.logger.log(
            LogLevel::Debug,
            format!("DNS CONFIG: {:?}", self.dns_config),
        );
        if !self.stop_request.load(std::sync::atomic::Ordering::Relaxed) {
            self.logger
                .log(LogLevel::Status, "Server is already listening.");
            return;
        } else {
            self.logger.log(LogLevel::Status, "Start Listening.");
        }

        self.stop_request
            .store(false, std::sync::atomic::Ordering::Relaxed);
        let mut buffer = [0; 1024];
        while !self.stop_request.load(std::sync::atomic::Ordering::Relaxed) {
            match self.server_socket.recv_from(&mut buffer) {
                Ok((received_bytes, client_address)) => {
                    let requested_domain = String::from_utf8_lossy(&buffer[..received_bytes]);
                    let ip = self.resolve_dns(&requested_domain);
                    self.server_socket
                        .send_to(ip.as_bytes(), &client_address)
                        .unwrap();
                }
                Err(_) => continue, // Handle timeout or other errors
            }
        }
        self.stop_request
            .store(true, std::sync::atomic::Ordering::Relaxed);
        self.logger.log(LogLevel::Status, "Stop Listening.");
    }

    pub fn processing_request(&mut self) {
        if self.stop_request.load(std::sync::atomic::Ordering::Relaxed) {
            return;
        };
        let mut buffer = [0; 1024];
        match self.server_socket.recv_from(&mut buffer) {
            Ok((received_bytes, client_address)) => {
                let requested_domain = String::from_utf8_lossy(&buffer[..received_bytes]);
                let ip = self.resolve_dns(&requested_domain);
                self.server_socket
                    .send_to(ip.as_bytes(), &client_address)
                    .unwrap();
            }
            Err(_) => return, // Handle timeout or other errors
        }
    }

    pub fn processing_command(&mut self, stdin_channel: &mut std::sync::mpsc::Receiver<String>) {
        let mut input = String::new();

        match stdin_channel.try_recv() {
            Ok(key) => {
                input = clean_io(&key);
            }
            Err(TryRecvError::Empty) => {}
            Err(TryRecvError::Disconnected) => panic!("Channel disconnected"),
        }

        if input == "stop" {
            self.logger.log(LogLevel::Warning, "Stop Listening");
            self.stop();
        } else if input == "start" {
            self.logger.log(LogLevel::Warning, "Start Listening");
            self.start();
        } else if input == "exit" {
            self.logger.log(LogLevel::Debug, "DNS Server Exited");
            self.exit();
        } else if input != "" {
            self.logger.log(
                LogLevel::Debug,
                format!("Unknown Command, Receving {}", input),
            );
        }
    }

    pub fn stop(&mut self) {
        self.stop_request
            .store(true, std::sync::atomic::Ordering::Relaxed);
    }

    pub fn start(&mut self) {
        self.stop_request
            .store(false, std::sync::atomic::Ordering::Relaxed);
    }

    pub fn exit(&mut self) {
        self.exit.store(true, std::sync::atomic::Ordering::Relaxed);
    }

    pub fn is_exited(&mut self) -> bool {
        self.exit.load(std::sync::atomic::Ordering::Relaxed)
    }

    pub fn run_processing_request(arc_mutex_dns: &Arc<Mutex<Self>>) {
        let cloned_dns = Arc::clone(&arc_mutex_dns);
        let cloned_dns_for_add_handle = Arc::clone(&arc_mutex_dns);
        let handle = thread::spawn(move || loop {
            let mut dns_server_copy = cloned_dns.lock().unwrap();
            if dns_server_copy.is_exited() {
                break;
            }
            dns_server_copy.processing_request();
        });

        cloned_dns_for_add_handle.lock().unwrap().add_handle(handle);
    }

    pub fn run_processing_command(arc_mutex_dns: &Arc<Mutex<Self>>) {
        let mut stdin_channel: std::sync::mpsc::Receiver<String> =
            crate::utils::spawn_stdin_channel();
        let cloned_dns = Arc::clone(&arc_mutex_dns);
        let cloned_dns_for_add_handle = Arc::clone(&arc_mutex_dns);
        let handle = thread::spawn(move || loop {
            let mut dns_server_copy = cloned_dns.lock().unwrap();
            if dns_server_copy.is_exited() {
                break;
            }
            dns_server_copy.processing_command(&mut stdin_channel);
            io::stdout().flush().unwrap();
        });

        cloned_dns_for_add_handle.lock().unwrap().add_handle(handle);
    }

    pub fn wait_exit(arc_mutex_dns: Arc<Mutex<Self>>) {
        let mut handles = vec![];
        {
            let mut dns_server_guard = arc_mutex_dns.lock().unwrap();
            handles.append(&mut dns_server_guard.handles);
        }
        for handle in handles {
            handle.join().unwrap();
        }
    }
}

impl fmt::Display for DNSServer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "DNS Server: {{ Name: {}, Port: {}, Remote: {}, Config File: {} }}",
            self.name, self.port, self.remote_addr, self.config_file_path
        )
    }
}

impl fmt::Debug for DNSServer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}
