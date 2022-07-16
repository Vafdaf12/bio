use std::io;

use crossterm::{
    event::{Event, KeyCode, KeyEvent, KeyModifiers},
    style::Stylize,
};

use crate::{raw::output::{QueueOperation, RawOutput}, raw::event::EventIter};
use super::process::{Process, ProcessEvent};

pub struct BetterCLI<'a> {
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

            for event in EventIter::default() {
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
