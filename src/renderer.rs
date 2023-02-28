use crate::{frame::Frame, NUM_COLS, NUM_ROWS};
use crossterm::{cursor, style, terminal, QueueableCommand};
use std::io::{Stdout, Write};

const BLOCK_CHAR: char = '▓';

fn number_to_styled_content(number: &i8) -> style::StyledContent<char> {
    let mut content_style = style::ContentStyle::new();
    content_style.foreground_color = match number {
        0 => Some(style::Color::Blue),
        1 => Some(style::Color::Yellow),
        2 => Some(style::Color::Green),
        3 => Some(style::Color::Red),
        4 => Some(style::Color::Cyan),
        5 => Some(style::Color::Magenta),
        6 => Some(style::Color::Grey),  // Exploding
        _ => Some(style::Color::Black), // None
    };
    style::StyledContent::new(content_style, BLOCK_CHAR)
}

#[derive(Debug)]
pub enum RendererError {
    Size,
    MinimumSize(usize, usize),
}

pub fn assert_screen_size() -> Result<(), RendererError> {
    let result = terminal::size().or(Err(RendererError::Size));

    if let Ok((cols, rows)) = result {
        if cols < NUM_COLS as u16 || rows < NUM_ROWS as u16 {
            return Err(RendererError::MinimumSize(NUM_COLS, NUM_ROWS));
        }
    } else {
        return Err(result.unwrap_err());
    }

    Ok(())
}

pub fn init(stdout: &mut Stdout, frame: &Frame) -> crossterm::Result<()> {
    stdout
        .queue(style::SetBackgroundColor(style::Color::AnsiValue(103)))?
        .queue(terminal::Clear(terminal::ClearType::All))?
        .queue(style::SetForegroundColor(style::Color::Black))?
        .queue(style::SetBackgroundColor(style::Color::Black))?;

    for (x, col) in frame.iter().enumerate() {
        for (y, number) in col.iter().enumerate() {
            stdout
                .queue(cursor::MoveTo(x as u16, y as u16))?
                .queue(style::Print(number_to_styled_content(number)))?;
        }
    }

    Ok(())
}

pub fn render(stdout: &mut Stdout, last_frame: &Frame, frame: &Frame) -> crossterm::Result<()> {
    for (x, col) in frame.iter().enumerate() {
        for (y, number) in col.iter().enumerate() {
            if last_frame[x][y] == frame[x][y] {
                continue;
            }
            stdout
                .queue(cursor::MoveTo(x as u16, y as u16))?
                .queue(style::Print(number_to_styled_content(number)))?;
        }
    }

    stdout.flush()?;

    Ok(())
}
