use anyhow::{anyhow, Result};
use clap::Parser;
use mshot_proto::screenshot::screenshot_handler_client::ScreenshotHandlerClient;
use mshot_proto::screenshot::Screenshot;
use std::fs;
use std::path::PathBuf;
use tokio::task::JoinSet;
use tonic::transport::{Endpoint, Error};

/// Command line interface
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Sets a custom output path (default: cwd)
    #[arg(short, long, value_name = "DIR")]
    out: Option<PathBuf>,

    /// Targets to take screenshots of
    #[arg(value_name = "NAME:IP[:PORT]", value_parser = parse_target)]
    targets: Vec<Target>,
}

/// A target to take a screenshot of
#[derive(Clone, Debug)]
struct Target {
    name: String,
    endpoint: Endpoint,
}

/// Parse a target
fn parse_target(input: &str) -> Result<Target, String> {
    let (name, ip) = input
        .split_once(|c| c == ':')
        .ok_or("Missing delimiter ':' between name and ip")?;
    let endpoint: Endpoint = ip.parse().map_err(|err: Error| err.to_string())?;
    Ok(Target {
        name: name.to_string(),
        endpoint,
    })
}

/// Main function
#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Check if we have targets
    if cli.targets.is_empty() {
        return Err(anyhow!("No targets specified"));
    }

    // Spawn screenshot tasks
    let mut set = JoinSet::new();
    for target in cli.targets {
        set.spawn(take_screenshot(target));
    }

    // Collect screenshots
    let mut screenshots = Vec::new();
    while let Some(result) = set.join_next().await {
        screenshots.push(result?);
    }

    // Save screenshots
    let output = cli.out.unwrap_or_else(|| ".".into());
    for screenshot in screenshots {
        let (
            name,
            Screenshot {
                width: _width,
                height: _height,
                image,
            },
        ) = screenshot?;
        let path = output.join(format!("{}.png", name));
        fs::write(&path, image)?;
        println!("Saved screenshot of {} to {}", name, path.display());
    }

    Ok(())
}

/// Take a screenshot of a target
async fn take_screenshot(target: Target) -> Result<(String, Screenshot)> {
    let Target { name, endpoint } = target;
    let mut client = ScreenshotHandlerClient::connect(endpoint).await?;
    let response = client.primary(()).await?;
    Ok((name, response.into_inner()))
}
