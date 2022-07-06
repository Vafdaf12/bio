use std::{
    io::{BufRead, BufReader, Write, self},
    process::{Child, Command, Stdio, ChildStdin, ExitStatus},
    sync::mpsc::{self, Receiver, Sender},
    thread,
};

pub enum ProcessEvent {
    Terminate,
    Output(String),
    Input(String)
}

pub struct EventIter<'a> {
    event_handle: &'a Receiver<ProcessEvent>
}

impl Iterator for EventIter<'_> {
    type Item = ProcessEvent;

    fn next(&mut self) -> Option<Self::Item> {
        match self.event_handle.try_recv() {
            Ok(msg) => Some(msg),
            Err(_) => None,
        }
    }
}

pub struct Process {
    pub input_buffer: String,
    process: Child,
    stdin: ChildStdin,
    event_receiver: Receiver<ProcessEvent>,
    event_sender: Sender<ProcessEvent>,
}

impl Process {
    pub fn new(command: &String) -> io::Result<Self> {
        let args: Vec<&str> = command.split_whitespace().collect();
        let mut process = Command::new(&args[0])
            .args(&args[1..])
            .stderr(Stdio::piped())
            .stdout(Stdio::piped())
            .stdin(Stdio::piped())
            .spawn()?;

        // Set up communication channels
        let (t_main, stdout_handle) = mpsc::channel::<ProcessEvent>();
        let msg_handle = t_main.clone();

        // Spawn stdout thread
        let reader = BufReader::new(process.stdout.take().unwrap());    
        thread::spawn(move || {
            for line in reader.lines().filter_map(|f| f.ok()) {
                t_main.send(ProcessEvent::Output(line)).unwrap();
            }
            t_main.send(ProcessEvent::Terminate).unwrap();
        });

        
        let stdin = process.stdin.take().unwrap();

        Ok(Process {
            process,
            input_buffer: String::new(),
            event_receiver: stdout_handle,
            stdin,
            event_sender: msg_handle,
        })
    }

    pub fn iter_events(&self) -> EventIter {
        EventIter {
            event_handle: &self.event_receiver,
        }
    }

    pub fn flush_stdin(&mut self) -> io::Result<()> {
        
        write!(self.stdin, "{}\n", self.input_buffer)?;
        self.stdin.flush()?;

        self.event_sender
            .send(ProcessEvent::Input(self.input_buffer.clone()))
            .unwrap();
        self.input_buffer.clear();

        Ok(())
    }
    pub fn kill(&mut self) -> io::Result<()> {
        self.process.kill()
    }

    pub fn wait(&mut self) -> io::Result<ExitStatus> {
        self.process.wait()
    }
}
