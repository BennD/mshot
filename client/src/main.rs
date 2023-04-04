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
    #[arg(value_name = "NAME:ENDPOINT", value_parser = parse_target)]
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
    let (name, endpoint) = input
        .split_once(|c| c == ':')
        .ok_or("Missing delimiter ':' between name and endpoint")?;
    Ok(Target {
        name: name.to_string(),
        endpoint: endpoint.parse().map_err(|err: Error| err.to_string())?,
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

    // Create output path
    let out = {
        let mut out = cli.out.clone().unwrap_or_else(|| PathBuf::from("."));
        out.push(chrono::Local::now().format("%Y-%m-%d_%H-%M-%S").to_string());
        out
    };

    // Spawn screenshot tasks into a JoinSet which can later be collected into a Vec
    let mut join_set = JoinSet::new();
    for target in cli.targets {
        let Target { name, endpoint } = target;

        // Spawn screenshot task
        join_set.spawn(async move {
            // Create client
            let mut client = ScreenshotHandlerClient::connect(endpoint)
                .await
                .map_err(|err| anyhow!("Failed to connect to {}: {}", name, err))?;

            // Take screenshot
            let screenshot = client
                .primary(())
                .await
                .map_err(|err| anyhow!("Failed to take screenshot of {}: {}", name, err))?
                .into_inner();

            // Return name / screenshot
            Ok::<(String, Screenshot), anyhow::Error>((name, screenshot))
        });
    }

    // Collect screenshots
    let mut screenshots = Vec::new();
    while let Some(screenshot) = join_set.join_next().await {
        screenshots.push(screenshot);
    }

    // Ensure output path exists
    fs::create_dir_all(&out)?;

    // Save screenshots to disk
    for screenshot in screenshots {
        // Unwrap screenshot
        let (name, screenshot) =
            screenshot.map_err(|err| anyhow!("Screenshot task failed: {}", err))??;

        // Write screenshot to disk with .png extension
        let mut target_path = out.clone();
        target_path.push(&name);
        target_path.set_extension("png");
        println!("Saving screenshot of {} to {}", name, target_path.display());
        fs::write(target_path, screenshot.image)?;
    }

    Ok(())
}
