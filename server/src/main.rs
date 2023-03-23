use anyhow::Result;
use screenshots::{DisplayInfo, Screen};
use tonic::{Request, Response, Status};
use tonic::transport::Server;
use mshot_proto::screenshot::Screenshot;
use mshot_proto::screenshot::screenshot_handler_server::{ScreenshotHandler, ScreenshotHandlerServer};

#[derive(Debug, Default)]
struct ScreenshotProvider;

#[tonic::async_trait]
impl ScreenshotHandler for ScreenshotProvider {
    async fn primary(&self, request: Request<()>) -> std::result::Result<Response<Screenshot>, Status> {
        println!("Got a request: {:?}", request);

        let screen = get_primary_screen().unwrap();
        let image = screen.capture().unwrap();
        let screenshot = Screenshot {
            width: image.width(),
            height: image.height(),
            image: image.into(),
        };

        Ok(Response::new(screenshot))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;
    let service = ScreenshotProvider::default();

    Server::builder()
        .add_service(ScreenshotHandlerServer::new(service))
        .serve(addr)
        .await?;

    Ok(())
}

fn get_primary_screen() -> Result<Screen> {
    let primary = DisplayInfo::all()?.into_iter().find(|info| info.is_primary).unwrap();
    Ok(Screen::new(&primary))
}