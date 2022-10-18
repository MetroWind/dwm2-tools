#![allow(non_snake_case)]

use std::io::prelude::*;
use std::fs::File;
use std::path::PathBuf;
use std::process::Command;

#[macro_use]
mod error;
mod breed;

use error::Error;

fn runDot(dot_code: &str, output_format: &str) -> Result<Vec<u8>, Error>
{
    let mut proc = Command::new("dot")
        .arg("-T")
        .arg(output_format)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| rterr!("Failed to run dot: {}", e))?;
    proc.stdin.take().unwrap().write_all(dot_code.as_bytes()).map_err(
        |e| rterr!("Failed to write input to dot: {}", e))?;
    let output = proc.wait_with_output().map_err(
        |e| rterr!("Failed to wait for dot to finish: {}", e))?;
    if !output.status.success()
    {
        return Err(rterr!("Dot command failed"));
    }

    Ok(output.stdout)
}

fn getOutputFileName(input_file: &str, output_type: &str) ->
    Result<String, Error>
{
    let path = PathBuf::from(input_file);
    let dir = if let Some(d) = path.parent()
    {
        d.to_owned()
    }
    else
    {
        std::env::current_dir().map_err(
            |e| rterr!("Failed to get current dir: {}", e))?
    };
    let basename: &str = path.file_stem().ok_or_else(
        || rterr!("Failed to get basename of input file"))?.to_str()
        .ok_or_else(|| rterr!("Input file name is too weird"))?;
    let output_basename_with_ext = format!("{}.{}", basename, output_type);
    let mut output_path = PathBuf::new();
    output_path.push(dir);
    output_path.push(output_basename_with_ext);
    Ok(output_path.to_str().ok_or_else(
        || rterr!("Output file name is too weird"))?.to_owned())
}

fn main() -> Result<(), Error>
{
    let options = clap::Command::new("DWM planner").arg(
        clap::Arg::new("FILE").required(true)
            .help("Input breed plan file")).arg(
        clap::Arg::new("output").short('o').long("output")
            .value_name("FILE")
            .help("Output file")).arg(
        clap::Arg::new("output_type").short('t').long("output-type")
            .value_name("TYPE").default_value("pdf")
            .help("Output file type. E.g. 'pdf', 'svg', etc. \
                   This argument is passed to the dot -T option."))
        .get_matches();

    let filename = options.get_one::<String>("FILE").unwrap();

    let f = std::fs::File::open(filename).map_err(|e| rterr!("{}", e))?;
    let output_file = if let Some(output) = options.get_one::<String>("output")
    {
        String::from(output)
    }
    else
    {
        getOutputFileName(
            filename, options.get_one::<String>("output_type").unwrap())?
    };

    let plan = breed::BreedPlan::fromStream(&mut std::io::BufReader::new(f))?;
    let output = runDot(
        &plan.toDot(), options.get_one::<String>("output_type").unwrap())?;
    let mut f = File::create(output_file).map_err(
        |e| rterr!("Failed to open output file: {}", e))?;
    f.write_all(&output).map_err(
        |e| rterr!("Failed to write output file: {}", e))?;

    Ok(())
}
