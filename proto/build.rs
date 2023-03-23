use std::io::Result;

fn main() -> Result<()> {
    tonic_build::configure()
        .out_dir("src")
        .include_file("lib.rs")
        .compile(&["proto/screenshot.proto"], &["proto"])?;
    Ok(())
}