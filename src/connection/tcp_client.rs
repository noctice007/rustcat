use std::{
    io::{BufRead, BufReader, Write},
    net::TcpStream,
    thread,
};

use crate::config::Config;
pub struct TcpClient<'a> {
    config: &'a Config,
}

impl<'a> TcpClient<'a> {
    pub fn new(config: &'a Config) -> Self {
        Self { config }
    }
    pub fn start(&self) -> anyhow::Result<()> {
        if let Some(ref target) = self.config.target {
            log::trace!("Connecting to {}:{}", target, self.config.port);
            let stream = TcpStream::connect(format!("{}:{}", target, self.config.port))?;
            let cloned_stream = stream.try_clone()?;
            thread::scope(move |s| {
                s.spawn(move || {
                    self.handle_reading(stream);
                });
                self.handle_writing(cloned_stream);
            });
            Ok(())
        } else {
            Err(anyhow::anyhow!("Target is not specified"))
        }
    }

    fn handle_reading(&self, mut stream: TcpStream) {
        loop {
            let buf_reader = BufReader::new(&mut stream);
            buf_reader.lines().for_each(|line| {
                println!("{}", line.unwrap());
            });
        }
    }

    fn handle_writing(&self, mut stream: TcpStream) {
        loop {
            let mut input = String::new();
            std::io::stdin().read_line(&mut input).unwrap();
            stream.write_all(input.as_bytes()).unwrap();
            stream.flush().unwrap();
        }
    }
}
