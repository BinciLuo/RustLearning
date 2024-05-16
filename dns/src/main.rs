mod cache;
mod utils;
use log;

fn main() {
    // Test log
    let logger = log::Logger::new("MyLogger", log::LogLevel::Debug);
    logger.log(log::LogLevel::Warning, &logger);
    logger.log(log::LogLevel::Fatal, logger.get_log_level());
    logger.log(log::LogLevel::Info, "This is an info message.");
    logger.log(log::LogLevel::Debug, "This is a debug message.");
    logger.log(log::LogLevel::Error, "This is an error message.");

    // Test utils
    let json_data = utils::read_json_file("config.json").expect("Failed to read JSON file");
    logger.log(log::LogLevel::Debug, format!("JSON data: {:?}", json_data));
    
    let shell_output =
        utils::exec_shell_command("echo Hello, World!").expect("Failed to execute shell command");
    println!("Shell command output: {}", shell_output);

    // Test cache
    let cache = cache::Cache::new(10);
    
}
