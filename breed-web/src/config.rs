use serde::Deserialize;

fn defaultListenAddr() -> String
{
    String::from("127.0.0.1")
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
}

impl Default for Configuration
{
    fn default() -> Self
    {
        Self {
            data_dir: String::from("."),
            listen_address: String::from("127.0.0.1"),
            listen_port: 8080,
        }
    }
}
