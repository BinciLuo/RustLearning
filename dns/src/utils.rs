use serde_json::Value;
use std::fs::File;
use std::io::Read;
use std::process::Command;
use reqwest;

#[derive(Debug)]
pub(crate) struct DnsResponse {
    // 定义你需要提取的DNS响应字段
    // 这里假设只关注A记录的IP地址
    pub ip_address: String,
}

impl DnsResponse {
    // 解析DNS响应
    pub fn from_json(json: serde_json::Value) -> Result<DnsResponse, Box<dyn std::error::Error>> {
        let ip_address = json["Answer"][0]["data"].as_str().unwrap_or("Unknown").to_string();
        Ok(DnsResponse { ip_address })
    }
}

pub fn _read_json_file(file_path: &str) -> Result<Value, io::Error> {
    let mut file = File::open(file_path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    let json_data: Value = serde_json::from_str(&contents)?;
    Ok(json_data)
}

use std::io::{self, ErrorKind};
use std::sync::mpsc::{self, Receiver};
use std::thread;

pub fn exec_shell_command(cmd: &str) -> Result<String, io::Error> {
    let output = Command::new("bash").arg("-c").arg(cmd).output()?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    } else {
        Err(io::Error::new(
            ErrorKind::Other,
            format!(
                "Command failed with status code {}: {}",
                output.status,
                String::from_utf8_lossy(&output.stderr)
            ),
        ))
    }
}

pub fn query_google_dns(domain: &str, record_type: &str) -> Result<DnsResponse, Box<dyn std::error::Error>> {
    let url = format!("https://dns.google/resolve?name={}&type={}&do=1", domain, record_type);
    let response_text = reqwest::blocking::get(&url)?.text()?;
    let json: serde_json::Value = serde_json::from_str(&response_text)?;
    let dns_response = DnsResponse::from_json(json)?;
    Ok(dns_response)
}

pub fn clean_io(origin: &str) -> String {
    // 移除空白字符和非打印字符
    let cleaned_domain = origin
        .trim()
        .chars()
        .filter(|c| c.is_ascii_graphic())
        .collect::<String>();
    cleaned_domain
}

pub fn spawn_stdin_channel() -> Receiver<String> {
    let (tx, rx) = mpsc::channel::<String>();
    thread::spawn(move || loop {
        let mut buffer = String::new();
        io::stdin().read_line(&mut buffer).unwrap();
        tx.send(buffer).unwrap();
    });
    rx
}
