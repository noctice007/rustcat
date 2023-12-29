use clap::Parser;
mod config;
mod connection;
mod rustcat;

fn main() -> anyhow::Result<()> {
    let config = config::Config::parse();
    let app = rustcat::RustCat::new(config);
    app.run()?;
    Ok(())
}
