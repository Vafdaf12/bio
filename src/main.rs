use std::io;
use std::process::Stdio;
use std::{env::args, process::Command};

use better_io::BetterOutput;

fn main() -> io::Result<()> {
    let args: Vec<String> = args().skip(1).collect();

    let mut child = Command::new(&args[0])
        .args(args[1..].iter())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let bio = BetterOutput::new(&mut child);
    'main: loop {
        if let Ok(event) = bio.next_event(&mut child) {
            match event {
                better_io::BioEvent::Terminated(status) => {
                    println!("Process exited with exit code {}", status.code().unwrap());
                    break 'main;
                }
                better_io::BioEvent::Output(o) => println!("OUT: {}", o),
                better_io::BioEvent::Error(e) => println!("ERR: {}", e),
            }
        }
    }

    Ok(())
}
