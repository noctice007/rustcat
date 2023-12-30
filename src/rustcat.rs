use crate::connection::tcp_client;
use std::io::Write;

use super::config::Config;
use super::connection::tcp_server;
pub struct RustCat {
    config: Config,
}

impl RustCat {
    pub fn new(config: Config) -> Self {
        Self { config }
    }
    pub fn run(&self) -> anyhow::Result<()> {
        let mut builder = env_logger::Builder::from_default_env();
        builder.format(|buf, record| writeln!(buf, "{}", record.args()));
        if self.config.verbose {
            builder.filter_level(log::LevelFilter::Trace);
        } else {
            builder.filter_level(log::LevelFilter::Info);
        }
        builder.init();
        if self.config.listen {
            self.start_server()
        } else {
            self.start_client()
        }
    }
    fn start_server(&self) -> anyhow::Result<()> {
        let server = tcp_server::TcpServer::new(&self.config);
        log::trace!("Starting server");
        server.start()
    }
    fn start_client(&self) -> anyhow::Result<()> {
        let client = tcp_client::TcpClient::new(&self.config);
        client.start()
    }
}
