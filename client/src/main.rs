use std::fs;
use mshot_proto::screenshot::screenshot_handler_client::ScreenshotHandlerClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = ScreenshotHandlerClient::connect("http://[::1]:50051").await?;

    let response = client.primary(()).await?;

    println!("RESPONSE={:?}", response);
    fs::write("got_it.png", response.into_inner().image).unwrap();

    Ok(())
}
