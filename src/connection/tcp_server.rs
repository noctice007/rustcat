use crate::config::Config;
use std::io::{stdin, BufRead, BufReader, Write};
use std::net::{Ipv4Addr, TcpListener, TcpStream};
use std::process::{ChildStdin, ChildStdout, Command, Stdio};
// use std::process::Command;
use std::thread;

pub struct TcpServer<'a> {
    config: &'a Config,
}

impl<'a> TcpServer<'a> {
    pub fn new(config: &'a Config) -> Self {
        Self { config }
    }
    pub fn start(&self) -> anyhow::Result<()> {
        let server = TcpListener::bind((Ipv4Addr::UNSPECIFIED, self.config.port))?;
        let (stream, address) = server.accept()?;
        println!("Got a connection from {:?}", address);
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
        let stdout = cmd.stdout.take().expect("Failed to initiate stdout");
        let stdin = cmd.stdin.take().expect("Failed to initiate stdin");
        let cloned_stream = stream.try_clone().expect("Could not clone TcpStream");
        thread::scope(move |s| {
            s.spawn(move || {
                self.handle_process_reading(cloned_stream, stdin);
            });
            self.handle_process_writing(stream, stdout);
        });
        cmd.wait().unwrap();
    }
    fn handle_process_reading(&self, mut stream: TcpStream, mut stdin: ChildStdin) {
        loop {
            let buf_reader = BufReader::new(&mut stream);
            buf_reader.lines().for_each(|line| {
                stdin
                    .write_all(format!("{}\n", line.unwrap()).as_bytes())
                    .unwrap();
                stdin.flush().unwrap();
            });
        }
    }
    fn handle_process_writing(&self, mut stream: TcpStream, mut stdout: ChildStdout) {
        loop {
            let buf_reader = BufReader::new(&mut stdout);
            buf_reader.lines().for_each(|line| {
                stream
                    .write_all(format!("{}\n", line.unwrap()).as_bytes())
                    .unwrap();
                stream.flush().unwrap();
            });
        }
    }

    fn handle_front_stream(&self, stream: TcpStream) {
        let cloned_stream = stream.try_clone().expect("Could not clone TcpStream");
        thread::scope(move |s| {
            s.spawn(move || {
                self.handle_reading(cloned_stream);
            });
            self.handle_writing(stream);
        });
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
            stdin().read_line(&mut input).unwrap();
            stream.write_all(input.as_bytes()).unwrap();
            stream.flush().unwrap();
        }
    }
}
