use anyhow::{bail, Result};
use discord_ci_cd::ci_cd::{Pipelines, PIPELINE_FILE};
use std::fs::read_to_string;
use std::process::Command;
use std::{env, path::PathBuf};
// use tokio::fs::read_to_string;

// #[tokio::main]
fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    let pipeline = if args.len() >= 2 {
        args[1].clone()
    } else {
        bail!("missing pipeline argument");
    };

    let mut pipeline_file: PathBuf = PathBuf::from("/home/dcicd-runner/repo/");
    pipeline_file.push(PIPELINE_FILE);
    let Ok(pipelines) =
        toml::from_str::<Pipelines>(&read_to_string(pipeline_file).unwrap_or(String::new()))
    else {
        bail!("failed to read {PIPELINE_FILE}");
    };

    // find pipline
    let Some(pipeline) = pipelines.get(&pipeline).map(|pl| pl.to_owned()) else {
        bail!("unknown pipeline: {pipeline}");
    };

    for cmd in pipeline.script {
        println!("$> {cmd}");
        let status = Command::new("sh").args(["-c", &cmd]).status()?;

        if !status.success() {
            let Some(code) = status.code() else {
                println!("command '{cmd}', was cancled by a signal.");

                break;
            };
            println!("command '{cmd}', exited with none-zero status '{code}'.");

            break;
        }
    }

    Ok(())
}
