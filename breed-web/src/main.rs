#![allow(non_snake_case)]

#[macro_use] extern crate error;
mod app;
mod config;

use error::Error;
use config::Configuration;

fn main() -> Result<(), Error>
{
    env_logger::Builder::default().format_timestamp(None)
        .filter_level(log::LevelFilter::Info)
        .init();

    let config = Configuration::default();
    let a = app::App::new(config)?;
    tokio::runtime::Runtime::new().unwrap().block_on(a.serve())?;
    Ok(())
}
