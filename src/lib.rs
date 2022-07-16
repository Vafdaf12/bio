pub mod error;
pub mod output;

use std::{
    io::{BufRead, BufReader},
    process::{Child, ExitStatus},
    sync::mpsc::{self, Receiver},
    thread::{self, JoinHandle},
};

use crate::error::BioResult;

/// Represents an I/O event that occurs on a child process.
///
/// Each event represents a line of data, e.g. an `Input(String)`
/// event only occurs if a complete line of of input is written
/// to the `stdin` of the child process
pub enum BioEvent {
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
        let mut bio = BetterOutput {
            event_recv,
            stdout_handle: None,
            stderr_handle: None,
        };

        if let Some(stdout) = child.stdout.take() {
            let sender_stdout = sender.clone();
            bio.stdout_handle = Some(thread::spawn(move || {
                let x = BufReader::new(stdout);

                x.lines()
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
    pub fn next_event(&self, child: &mut Child) -> BioResult<BioEvent> {
        match child.try_wait()? {
            Some(exit) => Ok(BioEvent::Terminated(exit)),
            None => Ok(self.event_recv.try_recv()?),
        }
    }
}
