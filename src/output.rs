use std::io::{stdout, Stdout, Write};
use std::{fmt, io};

use crossterm::terminal::{disable_raw_mode, enable_raw_mode, ClearType};
use crossterm::{execute, queue, style, terminal};
pub struct RawOutput {
    output: Stdout,
}

pub enum QueueOperation<T: fmt::Display> {
    Print(T),
    PrintLine(T),
}

impl RawOutput {
    pub fn init() -> io::Result<Self> {
        enable_raw_mode()?;
        Ok(RawOutput { output: stdout() })
    }

    pub fn flush(&mut self) -> io::Result<()> {
        self.output.flush()
    }

    pub fn queue<T: fmt::Display>(&mut self, op: QueueOperation<T>) -> io::Result<()> {
        match op {
            QueueOperation::Print(line) => queue!(
                self.output,
                terminal::Clear(ClearType::CurrentLine),
                style::Print(&format!("\r{}", line))
            ),
            QueueOperation::PrintLine(line) => queue!(
                self.output,
                terminal::Clear(ClearType::CurrentLine),
                style::Print(&format!("\r{}\n\r", line))
            ),
        }
    }

    pub fn println(&mut self, line: &str) -> io::Result<()> {
        execute!(
            self.output,
            terminal::Clear(ClearType::CurrentLine),
            style::Print(&format!("\r{}\n\r", line))
        )
    }

    pub fn print(&mut self, line: &str) -> io::Result<()> {
        execute!(
            self.output,
            terminal::Clear(ClearType::CurrentLine),
            style::Print(&format!("\r{}", line))
        )?;
        Ok(())
    }
}

impl Drop for RawOutput {
    fn drop(&mut self) {
        disable_raw_mode().unwrap();
    }
}
