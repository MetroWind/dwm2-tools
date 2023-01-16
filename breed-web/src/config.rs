use serde::Deserialize;

use error::Error;

fn defaultListenAddr() -> String
{
    String::from("127.0.0.1")
}

fn defaultServePath() -> String
{
    String::from("/")
}

fn defaultListenPort() -> u16 { 8080 }

#[derive(Deserialize, Clone)]
pub struct Configuration
{
    pub data_dir: String,
    #[serde(default = "defaultListenAddr")]
    pub listen_address: String,
    #[serde(default = "defaultListenPort")]
    pub listen_port: u16,
    #[serde(default = "defaultServePath")]
    pub serve_under_path: String,
}

impl Configuration
{
    pub fn fromFile(path: &str) -> Result<Self, Error>
    {
        let content = std::fs::read_to_string(path).map_err(
            |_| rterr!("Failed to read config file at {}", path))?;
        toml::from_str(&content).map_err(
            |_| rterr!("Failed to parse config file"))
    }
}

impl Default for Configuration
{
    fn default() -> Self
    {
        Self {
            data_dir: String::from("."),
            listen_address: String::from("127.0.0.1"),
            listen_port: 8080,
            serve_under_path: String::from("/"),
        }
    }
}
