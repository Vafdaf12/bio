use std::{fmt, io};

use crossterm::{
    cursor::MoveToColumn,
    queue,
    style::Print,
    terminal::{Clear, ClearType},
    QueueableCommand,
};

/// Clears the current line and moves the cursor to the start of the line
pub fn queue_clear_line<W: io::Write>(output: &mut W) -> io::Result<()> {
    queue!(output, MoveToColumn(0), Clear(ClearType::UntilNewLine))
}

/// Moves the cursor to the start of the next line
pub fn queue_newline<W: io::Write>(output: &mut W) -> io::Result<()> {
    output.queue(Print("\n\r"))?;
    Ok(())
}

/// Queues a print operation to the given writer
pub fn queue_print<D, W>(output: &mut W, line: &D) -> io::Result<()>
where
    D: fmt::Display,
    W: io::Write,
{
    output.queue(Print(line))?;
    Ok(())
}

/// Clears the line and draws to it, starting at the beginning of the line
pub fn queue_line<D, W>(output: &mut W, line: &D) -> io::Result<()>
where
    D: fmt::Display,
    W: io::Write,
{
    queue_clear_line(output)?;
    queue_print(output, line)?;
    queue_newline(output)
}
