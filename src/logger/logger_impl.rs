use std::fs::{self, OpenOptions};
use std::io::{Error, Write};
use std::sync::mpsc::{channel, Sender};
use std::thread;

/// Struct that represents the logger that will be used to log the info we need
pub struct Logger {
    sender: Sender<String>,
    _handle: thread::JoinHandle<()>,
}

impl Logger {
    pub fn new(dir: &str, file_name: &str) -> Result<Logger, Error> {
        // Create the directory if it doesn't exist
        fs::create_dir_all(dir).map_err(|e| Error::new(std::io::ErrorKind::AddrNotAvailable, e))?;

        let (sender, receiver) = channel();

        let file_name = format!("{}/{}.txt", dir, file_name);

        let handle = thread::spawn(move || {
            let mut file = match OpenOptions::new()
                .write(true)
                .create(true)
                .append(true)
                .open(&file_name)
            {
                Ok(file) => file,
                Err(e) => {
                    eprintln!("Failed to open log file: {}", e);
                    return;
                }
            };

            while let Ok(line) = receiver.recv() {
                if let Err(e) = writeln!(file, "{}", line) {
                    eprintln!("Failed to write to log file: {}", e);
                }
            }
        });

        Ok(Logger {
            sender,
            _handle: handle,
        })
    }

    pub fn log(&self, msg: &str) -> Result<(), Error> {
        self.sender
            .send(msg.to_string())
            .map_err(|e| Error::new(std::io::ErrorKind::Other, e))?;
        Ok(())
    }
}
