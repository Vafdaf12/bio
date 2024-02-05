use std::fs::File;
use std::io::{self, stdout, Write};
use std::process::Stdio;
use std::{env::args, process::Command};

use bio::regex_style::RegexStyle;
use bio::{output, BetterOutput, BioEvent};
use crossterm::event::{Event, KeyCode, KeyModifiers};
use crossterm::style::Stylize;

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

    let file = File::open("config.json")?;
    let rstyle: RegexStyle = serde_json::from_reader(file).expect("invalid JSON");

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

    draw_input(&mut stdout, &input)?;
    stdout.flush()?;

    'main: loop {
        if let Some(event) = bio.next_event(&mut child) {
            let event = event?;

            match event {
                BioEvent::Output(o) => {
                    output::queue_line(&mut stdout, &rstyle.style_stdout(&o))?;
                    draw_input(&mut stdout, &input)?;
                }
                BioEvent::Error(e) => {
                    output::queue_line(&mut stdout, &rstyle.style_stderr(&e))?;
                    draw_input(&mut stdout, &input)?;
                }

                BioEvent::Terminated(status) => {
                    let text = match status.code() {
                        Some(code) => format!("Process exited with code {}", code),
                        None => "Process exited abnormally".to_owned(),
                    };
                    output::queue_line(&mut stdout, &text)?;
                    break 'main;
                }

                BioEvent::Terminal(event) => {
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
                }
            };

            stdout.flush()?;
        }
    }
    stdout.flush()?;
    child.wait()?;

    Ok(())
}
