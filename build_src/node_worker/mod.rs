use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    io::{BufRead, BufReader, Error, ErrorKind, Result, Write},
    sync::{
        mpsc::{self, Receiver, SyncSender},
        RwLock,
    },
};
use std::{
    process::{Child, ChildStdout, Command, Stdio},
    sync::Arc,
    thread,
};

pub struct NodeWorker {
    process: Child,
    seq: u32,
    listeners: Arc<RwLock<HashMap<u32, SyncSender<WorkerResponse>>>>,
}

#[derive(Serialize, Debug)]
struct WorkerCommand<'a> {
    seq: u32,
    op: &'a str,
    data: &'a str,
}

#[derive(Deserialize)]
struct WorkerResponse {
    seq: u32,
    data: String,
}

fn receive_stdout(
    listeners: &RwLock<HashMap<u32, SyncSender<WorkerResponse>>>,
    out: ChildStdout,
) -> Result<()> {
    let reader = BufReader::new(out);

    for line in reader.lines() {
        let line = line?;

        let response = serde_json::from_str::<WorkerResponse>(&line).unwrap();
        if let Some(listener) = listeners.read().unwrap().get(&response.seq) {
            listener.send(response).unwrap();
        }
    }

    Ok(())
}

impl NodeWorker {
    pub fn new() -> Self {
        let process = if cfg!(windows) {
            Command::new("cmd")
                .arg("/C")
                .arg("node index.js")
                .current_dir("build_src/node_worker")
                .stdout(Stdio::piped())
                .stdin(Stdio::piped())
                .spawn()
                .unwrap()
        } else {
            Command::new("sh")
                .arg("-c")
                .arg("node index.js")
                .current_dir("build_src/node_worker")
                .stdout(Stdio::piped())
                .stdin(Stdio::piped())
                .spawn()
                .unwrap()
        };

        let mut worker = NodeWorker {
            process,
            seq: 0,
            listeners: Arc::new(RwLock::new(HashMap::new())),
        };
        worker.start_receiver();
        worker
    }

    fn start_receiver(&mut self) {
        let out = self.process.stdout.take().unwrap();
        let listeners = Arc::clone(&self.listeners);

        thread::spawn(move || {
            let map = listeners.as_ref();
            let _ = receive_stdout(map, out);
        });
    }

    fn setup_listener(&mut self) -> (u32, Receiver<WorkerResponse>) {
        let seq = {
            self.seq += 1;
            self.seq
        };

        let (tx, rx) = mpsc::sync_channel(24);
        self.listeners.write().unwrap().insert(seq, tx);

        (seq, rx)
    }

    fn send_command<'a>(&mut self, seq: u32, op: &'a str, data: &'a str) -> Result<()> {
        let cmd = WorkerCommand { seq, op, data };
        let command_json = serde_json::to_string(&cmd).unwrap();

        let stdin = self.process.stdin.as_mut().unwrap();
        writeln!(stdin, "{}", command_json)?;

        Ok(())
    }

    pub fn minify(&mut self, html: &str) -> Result<String> {
        let (seq, rx) = self.setup_listener();
        self.send_command(seq, "minify", html)?;

        let s = rx
            .recv()
            .map_err(|_| Error::new(ErrorKind::Other, "Failed to receive data"))?
            .data;

        Ok(s)
    }

    pub fn highlight(&mut self, html: &str) -> Result<String> {
        let (seq, rx) = self.setup_listener();
        self.send_command(seq, "highlight", html)?;

        let s = rx
            .recv()
            .map_err(|_| Error::new(ErrorKind::Other, "Failed to receive data"))?
            .data;

        Ok(s)
    }
}

impl Drop for NodeWorker {
    fn drop(&mut self) {
        let _ = self.process.kill();
    }
}
