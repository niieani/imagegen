use anyhow::Result;
use clap::Parser;

#[tokio::main]
async fn main() -> Result<()> {
    let outputs = imagegen::run(imagegen::Cli::parse()).await?;
    for out in outputs {
        println!("{}", out.display());
    }
    Ok(())
}
