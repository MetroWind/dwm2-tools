#![allow(non_snake_case)]

#[macro_use] extern crate error;
mod skill_detail;
mod app;
mod config;

use std::path::Path;

use log::warn;

use error::Error;
use config::Configuration;

fn main() -> Result<(), Error>
{
    let opts = clap::Command::new("Breed web")
        .about("Dragon Warrior: Monsters 2 breed database web server")
        .arg(clap::Arg::new("serve-path")
             .long("serve-path")
             .short('p')
             .value_name("PATH")
             .help("Serve under PATH."))
        .arg(clap::Arg::new("config")
             .long("config")
             .short('c')
             .value_name("FILE")
             .default_value("/etc/breed-web.toml")
             .help("Path of config file."))
        .get_matches();

    env_logger::Builder::default().format_timestamp(None)
        .filter_level(log::LevelFilter::Info)
        .init();

    let config_path = opts.get_one::<String>("config").unwrap();
    let mut config = if Path::new(&config_path).exists()
    {
        Configuration::fromFile(&config_path)?
    }
    else
    {
        warn!("Config file not found. Using default config...");
        Configuration::default()
    };

    if let Some(p) = opts.get_one::<String>("serve-path")
    {
        config.serve_under_path = p.clone();
    }
    let a = app::App::new(config)?;
    tokio::runtime::Runtime::new().unwrap().block_on(a.serve())?;
    Ok(())
}
