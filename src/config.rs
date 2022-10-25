use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::num::ParseIntError;
use thiserror::Error;

const PORT_LABEL: &str = "port";
const MAX_MEMORY_LABEL: &str = "max_memory";
const CONNECTION_BUFFER_SIZE_LABEL: &str = "connection_buffer_size";

#[derive(Debug)]
pub struct ServerConfig {
    port: u16,
    max_memory: u64,
    connection_buffer_size: usize,
}

#[derive(Debug, Error)]
pub enum ServerConfigError {
    #[error(transparent)]
    FileRead(#[from] io::Error),

    #[error("invalid config file format at '{0}'")]
    InvalidFormat(String),

    #[error("unknown directive '{0}' at '{1}'")]
    UnknownDirective(String, String),

    #[error(transparent)]
    ParseIntError(#[from] ParseIntError),
}

impl ServerConfig {
    pub fn load_from_disk(path: &str) -> Result<ServerConfig, ServerConfigError> {
        let reader = BufReader::new(File::open(path)?);
        Self::parse(reader)
    }

    fn parse(reader: BufReader<File>) -> Result<ServerConfig, ServerConfigError> {
        let mut config = ServerConfig {
            port: 1698,
            max_memory: 0,
            connection_buffer_size: 4096,
        };
        for maybe_line in reader.lines() {
            let line = &maybe_line?;
            if line.trim().starts_with('#') || line.trim().is_empty() {
                continue;
            }

            let tokens: Vec<&str> = line.split('=').map(|token| token.trim()).collect();

            if tokens.len() < 2 || tokens.len() > 2 {
                return Err(ServerConfigError::InvalidFormat(line.clone()));
            }

            match tokens[0] {
                PORT_LABEL => {
                    let port = tokens[1].parse::<u16>()?;
                    config.port = port;
                }
                MAX_MEMORY_LABEL => {
                    if tokens[1].len() < 3 {
                        return Err(ServerConfigError::InvalidFormat(line.clone()));
                    }
                    let unit = &tokens[1][tokens[1].len() - 2..];
                    let memory = tokens[1][..tokens[1].len() - 2].parse::<u64>()?;
                    match unit {
                        "mb" => config.max_memory = memory * 1024 * 1024,
                        "gb" => config.max_memory = memory * 1024 * 1024 * 1024,
                        _ => {
                            return Err(ServerConfigError::InvalidFormat(line.clone()));
                        }
                    }
                }
                CONNECTION_BUFFER_SIZE_LABEL => {
                    let connection_buffer_size = tokens[1].parse::<usize>()?;
                    config.connection_buffer_size = connection_buffer_size;
                }
                _ => {
                    return Err(ServerConfigError::UnknownDirective(
                        tokens[0].to_string(),
                        line.clone(),
                    ))
                }
            }
        }

        Ok(config)
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn max_memory(&self) -> u64 {
        self.max_memory
    }

    pub fn connection_buffer_size(&self) -> usize {
        self.connection_buffer_size
    }
}
