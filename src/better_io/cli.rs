use std::{
    io::{self, stdout, Stdout},
    time::Duration,
};

use crossterm::{
    event::{poll, read, Event, KeyCode, KeyEvent, KeyModifiers},
    style::Stylize,
};

use crate::raw_output::{QueueOperation, RawOutput};
use super::process::{Process, ProcessEvent};

pub struct BetterCLI<'a> {
    stdout: Stdout,
    process: &'a mut Process,
    event_format: fn(ProcessEvent) -> String,
}

fn passthrough(e: ProcessEvent) -> String {
    match e {
        ProcessEvent::Terminate => "Process Terminated".to_string(),
        ProcessEvent::Output(s) => s,
        ProcessEvent::Input(s) => s.cyan().to_string(),
    }
}

impl<'a> BetterCLI<'a> {
    pub fn new(process: &'a mut Process) -> BetterCLI<'a> {
        BetterCLI {
            stdout: stdout(),
            process,
            event_format: passthrough,
        }
    }
}

impl BetterCLI<'_> {
    pub fn set_event_format(&mut self, func: fn(ProcessEvent) -> String) {
        self.event_format = func;
    }

    pub fn run(&mut self) -> io::Result<()> {
        let mut output = RawOutput::init()?;

        'main: loop {
            let mut changed = false;

            for msg in self.process.iter_events() {
                match msg {
                    ProcessEvent::Terminate => break 'main,
                    e => output.queue(QueueOperation::PrintLine((self.event_format)(e)))?,
                }
            }

            for event in InputIter::new(50) {
                match event? {
                    Event::Key(e) => match e {
                        KeyEvent {
                            code: KeyCode::Char('c'),
                            modifiers: KeyModifiers::CONTROL,
                        } => {
                            output.queue(QueueOperation::PrintLine("Exiting Process..."))?;

                            self.process.kill()?;
                            break 'main;
                        }
                        KeyEvent {
                            code: KeyCode::Char(ch),
                            ..
                        } => {
                            self.process.input_buffer.push(ch);
                            changed |= true;
                        }

                        KeyEvent {
                            code: KeyCode::Backspace,
                            ..
                        } => {
                            self.process.input_buffer.pop();
                            changed |= true;
                        }

                        KeyEvent {
                            code: KeyCode::Enter,
                            ..
                        } => {
                            self.process.flush_stdin().unwrap();
                            changed |= true;
                        }
                        _ => {}
                    },
                    _ => {}
                }
            }

            if changed {
                output.queue(QueueOperation::Print(format!(
                    "> {}",
                    self.process.input_buffer
                )))?;
            }

            output.flush()?;
        }

        match self.process.wait()?.code() {
            Some(code) => output.println(&format!("Process exited with exit code {}", code))?,
            None => output.println("Process exited abnormally")?,
        }

        Ok(())
    }
}

/// The [InputIter] type serves as a helper
/// type that aids in event-handling when
/// raw mode is enabled in [crossterm]
struct InputIter {
    timeout: Duration,
}

impl InputIter {
    fn new(timeout: u64) -> Self {
        InputIter {
            timeout: Duration::from_millis(timeout),
        }
    }
}

impl Iterator for InputIter {
    type Item = io::Result<Event>;

    fn next(&mut self) -> Option<Self::Item> {
        match poll(self.timeout) {
            Err(e) => Some(Err(e)),
            Ok(m) => m.then(|| read()),
        }
    }
}
