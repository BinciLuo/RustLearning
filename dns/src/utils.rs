use serde_json::Value;
use std::fs::File;
use std::io::{self, Read};
use std::process::Command;

pub fn read_json_file(file_path: &str) -> Result<Value, io::Error> {
    let mut file = File::open(file_path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    let json_data: Value = serde_json::from_str(&contents)?;
    Ok(json_data)
}

pub fn exec_shell_command(cmd: &str) -> Result<String, io::Error> {
    let output = Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .output()?;

    if output.status.success() {
        let result = String::from_utf8_lossy(&output.stdout).into_owned();
        Ok(result)
    } else {
        let error_message = String::from_utf8_lossy(&output.stderr).into_owned();
        Err(io::Error::new(io::ErrorKind::Other, error_message))
    }
}
