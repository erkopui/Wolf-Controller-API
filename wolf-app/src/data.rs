use serde_json::Value;
use std::fs;

#[derive(Clone)]
pub struct App {
    pub data: Value,
}

impl App {
    pub fn new(conf_file: &str) -> Result<App, Box<dyn std::error::Error>> {
        let content = fs::read(conf_file)?;
        let data: Value = serde_json::from_slice(&content)?;
        Ok(App { data })
    }
}

