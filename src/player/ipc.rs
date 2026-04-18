use std::os::unix::net::UnixStream;
use std::io::{Read, Write};
use tracing::debug;

pub struct IpcClient {
    socket_path: String,
}

impl IpcClient {
    pub fn new(socket_path: &str) -> Self {
        Self {
            socket_path: socket_path.to_string(),
        }
    }

    pub fn send_command(&mut self, command: &str) -> Result<String, String> {
        let mut stream = UnixStream::connect(&self.socket_path)
            .map_err(|e| format!("Failed to connect to {}: {}", self.socket_path, e))?;

        let full_command = format!("{}\n", command);
        stream.write_all(full_command.as_bytes())
            .map_err(|e| format!("Failed to send command: {}", e))?;

        drop(stream);

        let mut stream = UnixStream::connect(&self.socket_path)
            .map_err(|e| format!("Failed to connect for response: {}", e))?;
        let mut response = String::new();
        stream.read_to_string(&mut response)
            .map_err(|e| format!("Failed to read response: {}", e))?;

        debug!("IPC response: {}", response);
        Ok(response)
    }
}