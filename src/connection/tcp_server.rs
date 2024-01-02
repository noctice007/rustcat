use crate::config::Config;
use std::io::{self, BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::os::fd::{AsRawFd, FromRawFd};
use std::process::{Command, Stdio};
use std::thread;

pub struct TcpServer<'a> {
    config: &'a Config,
}

impl<'a> TcpServer<'a> {
    pub fn new(config: &'a Config) -> Self {
        Self { config }
    }
    pub fn start(&self) -> anyhow::Result<()> {
        let server = TcpListener::bind(format!("{}:{}", self.config.source, self.config.port))?;
        log::trace!("Listening in {:?}", server.local_addr().unwrap());
        let (stream, address) = server.accept()?;
        log::trace!("Got a connection from {:?}", address);
        self.handle_stream(stream)?;
        Ok(())
    }
    fn handle_stream(&self, stream: TcpStream) -> anyhow::Result<()> {
        if let Some(ref cmd) = self.config.execute {
            self.handle_process_stream(cmd, stream)?;
        } else {
            self.handle_front_stream(stream);
        }
        Ok(())
    }

    fn handle_process_stream(&self, cmd: &String, stream: TcpStream) -> anyhow::Result<()> {
        let fd = stream.as_raw_fd();
        Command::new(cmd)
            .stdin(unsafe { Stdio::from_raw_fd(fd) })
            .stdout(unsafe { Stdio::from_raw_fd(fd) })
            .spawn()?
            .wait()?;
        Ok(())
    }

    fn handle_front_stream(&self, stream: TcpStream) {
        let cloned_stream = stream.try_clone().expect("Could not clone TcpStream");
        thread::scope(move |s| {
            s.spawn(move || {
                self.handle_stdout(cloned_stream, &mut io::stdout());
            });
            self.handle_stdin(stream, &mut io::stdin().lock());
        });
    }
    fn handle_stdout(&self, mut stream: TcpStream, stdout: &mut dyn Write) {
        loop {
            let buf_reader = BufReader::new(&mut stream);
            buf_reader.lines().for_each(|line| {
                stdout
                    .write_all(format!("{}\n", line.unwrap()).as_bytes())
                    .unwrap();
            });
        }
    }
    fn handle_stdin(&self, mut stream: TcpStream, stdin: &mut dyn BufRead) {
        loop {
            let mut input = String::new();
            stdin.read_line(&mut input).unwrap();
            stream.write_all(input.as_bytes()).unwrap();
            stream.flush().unwrap();
        }
    }
}
