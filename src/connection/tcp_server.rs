use crate::config::Config;
use std::io::{self, BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
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
            self.handle_process_stream(cmd, stream);
        } else {
            self.handle_front_stream(stream);
        }
        Ok(())
    }

    fn handle_process_stream(&self, cmd: &String, stream: TcpStream) {
        let mut cmd = Command::new(cmd.clone())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .expect(format!("Failed to start {}", cmd).as_str());
        let mut stdout = cmd.stdout.take().expect("Failed to initiate stdout");
        let mut stdin = cmd.stdin.take().expect("Failed to initiate stdin");
        let cloned_stream = stream.try_clone().expect("Could not clone TcpStream");
        thread::scope(move |s| {
            s.spawn(move || {
                let mut buf_reader = BufReader::new(&mut stdout);
                self.handle_stdin(cloned_stream, &mut buf_reader);
            });
            self.handle_stdout(stream, &mut stdin);
        });
        cmd.wait().unwrap();
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
