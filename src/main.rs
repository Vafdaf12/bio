use std::env::args;
use std::io;

use crossterm::{

    style::Stylize,
};

mod better_io;
mod raw_output;

use better_io::{
    cli::BetterCLI,
    process::{Process, ProcessEvent},
};

fn main() -> io::Result<()> {
    let args: Vec<String> = args().collect();
    if args.len() < 2 {
        println!("Please specify command");
        std::process::exit(1);
    }

    let mut process = Process::new(&args[1])?;
    let mut bcli = BetterCLI::new(&mut process);

    bcli.set_event_format(|ev| match ev {
        ProcessEvent::Output(s) => {
            let trimmed = s.trim();
            if trimmed.starts_with('[') && trimmed.ends_with(']') {
                
                s.yellow().bold().to_string()
            }
            else if trimmed.starts_with('#') {
                s.dark_grey().to_string()
            }
            else if trimmed.contains('=') {
                let mut splitted: Vec<String> = trimmed.split('=').map(|s| s.to_string()).collect();
                splitted[0] = splitted[0].clone().green().to_string();
                splitted.join("=")
            }
            else {
                s
            }
        },
        ProcessEvent::Input(s) => s,
        ProcessEvent::Terminate => "".to_string()
    });

    bcli.run()?;


    Ok(())
}
