use anyhow::{anyhow, Result};
use clap::Parser;
use mshot_proto::screenshot::screenshot_handler_server::{
    ScreenshotHandler, ScreenshotHandlerServer,
};
use mshot_proto::screenshot::Screenshot;
use screenshots::{DisplayInfo, Screen};
use std::net::{AddrParseError, SocketAddr};
use tonic::transport::Server;
use tonic::{Request, Response, Status};

/// The command line interface
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(value_parser = parse_addr)]
    address: SocketAddr,
}

/// Parse a socket address
fn parse_addr(input: &str) -> Result<SocketAddr, String> {
    input.parse().map_err(|err: AddrParseError| err.to_string())
}

/// The screenshot provider
#[derive(Debug, Default)]
struct ScreenshotProvider;

/// Implement the screenshot handler
#[tonic::async_trait]
impl ScreenshotHandler for ScreenshotProvider {
    async fn primary(
        &self,
        request: Request<()>,
    ) -> std::result::Result<Response<Screenshot>, Status> {
        println!("Got a request: {:?}", request);

        let screen = get_primary_screen()
            .map_err(|err| Status::internal(format!("Failed to get primary screen: {}", err)))?;
        let image = screen.capture().map_err(|err| {
            Status::internal(format!("Failed to capture primary screen: {}", err))
        })?;
        let screenshot = Screenshot {
            width: image.width(),
            height: image.height(),
            image: image.into(),
        };

        Ok(Response::new(screenshot))
    }
}

/// Main function
#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let addr = cli.address;
    let screenshot_provider = ScreenshotProvider::default();

    println!("Listening on {}", addr);

    Server::builder()
        .add_service(ScreenshotHandlerServer::new(screenshot_provider))
        .serve(addr)
        .await
        .map_err(|err| anyhow!("Failed to start server: {}", err))?;

    Ok(())
}

/// Get the primary screen
fn get_primary_screen() -> Result<Screen> {
    let primary = DisplayInfo::all()?
        .into_iter()
        .find(|info| info.is_primary)
        .ok_or_else(|| anyhow!("No primary display found"))?;
    Ok(Screen::new(&primary))
}
