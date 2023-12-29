use clap::Parser;
#[derive(Parser)]
pub struct Config {
    pub target: Option<String>,
    #[arg(long, short, help = "Start listen mode")]
    pub listen: bool,
    #[arg(long, short, help = "Specify the port", required = true)]
    pub port: u16,
    #[arg(long, short, help = "Execute a give command", requires = "listen")]
    pub execute: Option<String>,
}
