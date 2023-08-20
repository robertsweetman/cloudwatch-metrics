use serde::Deserialize;
use std::fs::File;
use std::io::Read;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub polling_interval_minutes: u64,
    pub region: String,
    pub process_checks: Vec<String>,
    pub script_paths: Vec<Script>,
}

#[derive(Debug, Deserialize)]
pub struct Script {
    pub path: String,
    pub user: String,
}

pub fn read_config(filename: &str) -> Result<Config, Box<dyn std::error::Error>> {
    let mut file = File::open(filename)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    let config: Config = serde_yaml::from_str(&contents)?;

    Ok(config)
}