use std::{fs::File, path::PathBuf};

use anyhow::Result;
use serde::{Deserialize, Serialize};

const CONFIG_NAME: &str = ".protonvpn-transmission-natpmp.cfg";

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
pub struct Config {
    pub gateway: String,

    #[serde(default)]
    pub transmission: Option<TransmissionRpcConfig>,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]

pub struct TransmissionRpcConfig {
    pub rpc_url: String,
    pub rpc_username: String,
    pub rpc_password: String,
}

impl Config {
    pub fn config_path() -> Result<PathBuf> {
        let mut exec_dir = std::env::current_exe()?;
        exec_dir.pop();
        Ok(exec_dir.join(CONFIG_NAME))
    }

    pub fn new() -> Result<Self> {
        let config_file = File::open(Self::config_path()?)?;
        Ok(serde_json::from_reader(config_file)?)
    }
}
