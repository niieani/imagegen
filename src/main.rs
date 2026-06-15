use anyhow::Result;
use clap::Parser;

#[tokio::main]
async fn main() -> Result<()> {
    let out = imagegen::run(imagegen::Cli::parse()).await?;
    println!("{}", out.display());
    Ok(())
}
