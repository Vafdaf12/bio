use std::io::{self, stdout, Write};
use std::process::Stdio;
use std::{env::args, process::Command};

use better_io::{BetterOutput, BioEvent, output};
use crossterm::event::{Event, KeyCode, KeyModifiers};
use crossterm::style::Stylize;

use termy::event::EventIter;
use termy::input::RawInput;
use termy::raw::RawMode;
use termy::Widget;

fn draw_input<W: io::Write>(writer: &mut W, input: &RawInput) -> io::Result<()> {
    output::queue_clear_line(writer)?;
    output::queue_print(writer, &"> ".dark_grey())?;
    input.draw(writer)?;
    Ok(())
}

fn main() -> io::Result<()> {
    let args: Vec<String> = args().skip(1).collect();

    let mut child = Command::new(&args[0])
        .args(args[1..].iter())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    let mut child_stdin = child
        .stdin
        .take()
        .expect("failed to retrieve process stdin");

    let bio = BetterOutput::new(&mut child);

    let _raw = RawMode::enable()?;
    let mut stdout = stdout();
    let mut input = RawInput::new();

    'main: loop {
        if let Some(event) = bio.next_event(&mut child) {
            let event = event?;

            // Process output
            let line = match &event {
                BioEvent::Terminated(status) => match status.code() {
                    Some(code) => format!("Process exited with code {}", code),
                    None => "Process exited abnormally".to_owned(),
                },
                BioEvent::Output(o) => format!("{}: {}", "OUT".bold(), o),
                BioEvent::Error(e) => format!("{}: {}", "ERR".bold().red(), e),
            };
            output::queue_line(&mut stdout, &line)?;

            // Event handling
            match event {
                BioEvent::Terminated(_) => break 'main,
                _ => {}
            }
            draw_input(&mut stdout, &input)?;
            stdout.flush()?;
        }

        for event in EventIter::default() {
            let event = event?;
            input.handle_event(&event);

            if let Event::Key(key) = event {
                match key.code {
                    KeyCode::Char('c') if key.modifiers == KeyModifiers::CONTROL => {
                        output::queue_line(&mut stdout, &"Stopping Process...")?;
                        child.kill()?;
                    }
                    KeyCode::Enter if !input.value().is_empty() => {
                        let value = input.value().to_owned();
                        child_stdin.write(format!("{}\n", value).as_bytes())?;
                        child_stdin.flush()?;

                        input.set_value("");
                    }
                    _ => {}
                }
            }

            draw_input(&mut stdout, &input)?;
            stdout.flush()?;
        }
    }
    stdout.flush()?;
    child.wait()?;

    Ok(())
}
