
#[cfg(test)]
use core::time;
#[cfg(test)]
use std::thread::sleep;
#[cfg(test)]
use super::*;

#[test]
fn test_log() {
    let logger = log::Logger::new("MyLogger", log::LogLevel::Debug);
    logger.log(log::LogLevel::Warning, &logger);
    logger.log(log::LogLevel::Fatal, logger.get_log_level());
    logger.log(log::LogLevel::Info, "This is an info message.");
    logger.log(log::LogLevel::Debug, "This is a debug message.");
    logger.log(log::LogLevel::Error, "This is an error message.");
}

#[test]
fn test_utils(){
    let logger = log::Logger::new("MyLogger", log::LogLevel::Debug);
    let json_data = utils::_read_json_file("../config.json").expect("Failed to read JSON file");
    logger.log(log::LogLevel::Debug, format!("JSON data: {:?}", json_data));

    let shell_output =
        utils::exec_shell_command("echo Hello, World!").expect("Failed to execute shell command");
    logger.log(log::LogLevel::Debug, shell_output.clone());
    assert_eq!(shell_output, "Hello, World!\n");
}

#[test]
fn test_cache(){
    let logger = log::Logger::new("MyLogger", log::LogLevel::Debug);
    let cache = cache::Cache::new(1);
    cache.put("binciluo".to_string(), "127.0.0.1".to_string());
    let key = "binciluo".to_string();
    if let Some(ip) = cache.get(key.clone().as_str(), false) {
        logger.log(log::LogLevel::Info, format!("Got {}---->{}", key, ip))
    } else {
        logger.log(log::LogLevel::Warning, format!("Unfound {}", key.clone()));
        panic!("Should be found.");
    };
    sleep(time::Duration::from_secs(1));
    if let Some(ip) = cache.get(key.clone().as_str(), false) {
        logger.log(log::LogLevel::Info, format!("Got {}---->{}", key, ip));
        panic!("Should not be found. It shoud be removed from cache after 1s.");
    } else {
        logger.log(log::LogLevel::Warning, format!("Unfound {}", key.clone()))
    };
}
