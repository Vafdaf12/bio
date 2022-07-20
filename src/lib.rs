pub mod output;
pub mod regex_style;

use std::{
    io::{self, BufRead, BufReader},
    process::{Child, ExitStatus},
    sync::mpsc::{self, Receiver},
    thread::{self, JoinHandle},
    time::Duration,
};

use crossterm::event::{read, Event};

/// Represents an I/O event that occurs on a child process.
///
/// Each event represents a line of data, e.g. an `Input(String)`
/// event only occurs if a complete line of of input is written
/// to the `stdin` of the child process
pub enum BioEvent {
    // A terminal event has occured
    Terminal(Event),

    /// The process has stopped
    Terminated(ExitStatus),

    /// A line has been written to the stdout of the child process
    Output(String),

    /// A line has been written to the stderr of the child process
    Error(String),
}

/// Provides an event-based API for the output
/// produced by a child process.
///
/// Keep in mind that the `stdout` and `stderr`
/// of the child process needs to be piped in order
/// for this API to work.
pub struct BetterOutput {
    stdout_handle: Option<JoinHandle<()>>,
    stderr_handle: Option<JoinHandle<()>>,

    event_recv: Receiver<BioEvent>,
}

impl BetterOutput {
    pub fn new(child: &mut Child) -> Self {
        let (sender, event_recv) = mpsc::channel::<BioEvent>();
        let sender_term = sender.clone();

        thread::spawn(move || loop {
            sender_term
                .send(BioEvent::Terminal(read().unwrap()))
                .unwrap();
        });

        let mut bio = BetterOutput {
            event_recv,
            stdout_handle: None,
            stderr_handle: None,
        };

        if let Some(stdout) = child.stdout.take() {
            let sender_stdout = sender.clone();
            bio.stdout_handle = Some(thread::spawn(move || {
                BufReader::new(stdout)
                    .lines()
                    .filter_map(|f| f.ok())
                    .map(|line| BioEvent::Output(line))
                    .for_each(|ev| {
                        sender_stdout.send(ev).unwrap();
                    });
            }));
        }

        if let Some(stderr) = child.stderr.take() {
            let sender_stderr = sender.clone();
            bio.stderr_handle = Some(thread::spawn(move || {
                BufReader::new(stderr)
                    .lines()
                    .filter_map(|f| f.ok())
                    .map(|line| BioEvent::Error(line))
                    .for_each(|ev| {
                        sender_stderr.send(ev).unwrap();
                    });
            }));
        }

        bio
    }

    /// Waits for the `stdout` and `stderr` threads to exit
    pub fn wait(&mut self) -> thread::Result<()> {
        if let Some(thread) = self.stdout_handle.take() {
            thread.join()?;
        }
        if let Some(thread) = self.stderr_handle.take() {
            thread.join()?;
        }

        Ok(())
    }

    /// Tries to get the next event and  check to see
    /// if the child process is exited
    pub fn next_event(&self, child: &mut Child) -> Option<io::Result<BioEvent>> {
        const TIMEOUT: Duration = Duration::from_secs(1);

        match self.event_recv.recv_timeout(TIMEOUT) {
            Ok(event) => Some(Ok(event)),
            Err(_) => match child.try_wait() {
                Ok(status) => status.map(|x| BioEvent::Terminated(x)).map(|x| Ok(x)),
                Err(e) => Some(Err(e)),
            },
        }
    }
}
