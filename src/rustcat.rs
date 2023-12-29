use crate::connection::tcp_client;

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
        if self.config.listen {
            self.start_server()
        } else {
            self.start_client()
        }
    }
    fn start_server(&self) -> anyhow::Result<()> {
        let server = tcp_server::TcpServer::new(&self.config);
        server.start()
    }
    fn start_client(&self) -> anyhow::Result<()> {
        let client = tcp_client::TcpClient::new(&self.config);
        client.start()
    }
}
