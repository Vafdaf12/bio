use std::io::{self, stdout, Write};
use std::process::Stdio;
use std::{env::args, process::Command};

use better_io::{BetterOutput, BioEvent};
use crossterm::cursor::MoveToColumn;
use crossterm::queue;
use crossterm::style::{Print, Stylize};
use crossterm::terminal::{Clear, ClearType};
use termy::raw::RawMode;

fn main() -> io::Result<()> {
    let args: Vec<String> = args().skip(1).collect();

    let mut child = Command::new(&args[0])
        .args(args[1..].iter())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let bio = BetterOutput::new(&mut child);

    let _raw = RawMode::enable()?;
    let mut stdout = stdout();

    'main: loop {
        if let Ok(event) = bio.next_event(&mut child) {
            let line = match &event {
                BioEvent::Terminated(status) => {
                    let code = status.code().unwrap();
                    format!("Process exited with code {}", code)
                }
                BioEvent::Output(o) => format!("{}: {}", "OUT".bold(), o),
                BioEvent::Error(e) => format!("{}: {}", "ERR".bold().red(), e),
            };
            queue!(
                stdout,
                MoveToColumn(1),
                Clear(ClearType::CurrentLine),
                Print(line),
                Print("\n\r")
            )?;
            stdout.flush()?;

            match event {
                BioEvent::Terminated(_) => break 'main,
                _ => {}
            }
        }
    }
    Ok(())
}
