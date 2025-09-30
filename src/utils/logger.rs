use chrono::Utc;
use log::{Level, LevelFilter, Log, Metadata, Record};
use std::fs::OpenOptions;
use std::io::Write;

pub struct DiscordLogger {
    file: std::fs::File,
}

impl DiscordLogger {
    pub fn new(log_file: &str) -> Result<DiscordLogger, std::io::Error> {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_file)?;
        
        Ok(DiscordLogger { file })
    }

    pub fn init(log_file: &str) -> Result<(), Box<dyn std::error::Error>> {
        let logger = DiscordLogger::new(log_file)?;
        log::set_boxed_logger(Box::new(logger))?;
        log::set_max_level(LevelFilter::Info);
        Ok(())
    }
}

impl Log for DiscordLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Info
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let timestamp = Utc::now().format("%Y-%m-%d %H:%M:%S%.3f").to_string();
            let log_entry = format!("[{}] {} - {}\n", 
                timestamp, 
                record.level(), 
                record.args()
            );
            
            // Write to file
            let mut file = self.file.try_clone().expect("Failed to clone file handle");
            let _ = file.write_all(log_entry.as_bytes());
            let _ = file.flush();
        }
    }

    fn flush(&self) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logger_creation() {
        let logger = DiscordLogger::new("test.log");
        assert!(logger.is_ok());
    }
}
