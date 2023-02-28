use crossterm::{
    cursor::{Hide, Show},
    event::{self, poll, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand, Result,
};
use rust_columns::{
    column::Column,
    frame::{new_frame, Drawable, Frame},
    pit::Pit,
    renderer::{self, assert_screen_size},
};
use std::{
    io,
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};

fn main() -> Result<()> {
    assert_screen_size().expect("Failed when asserting the screen size requirements");

    // Terminal
    let mut stdout = io::stdout();

    enable_raw_mode()?;
    stdout.execute(EnterAlternateScreen)?.execute(Hide)?;

    // Render loop in a separate thread
    let (render_tx, render_rx) = mpsc::channel::<Frame>();
    thread::spawn(move || -> Result<()> {
        let mut stdout = io::stdout();
        let mut last_frame = new_frame();
        renderer::init(&mut stdout, &last_frame)?;
        while let Ok(curr_frame) = render_rx.recv() {
            renderer::render(&mut stdout, &last_frame, &curr_frame)?;
            last_frame = curr_frame;
        }
        Ok(())
    });

    let fps_duration = Duration::from_nanos(1_000_000_000 / 60); // 60 fps duration ~16ms
    let mut column = Column::new();
    let mut upcoming_column = Column::new();
    let mut pit = Pit::new();
    let mut instant = Instant::now();

    'gameloop: loop {
        let delta = instant.elapsed();
        instant = Instant::now();
        let mut curr_frame = new_frame();

        while poll(Duration::default())? {
            if let Event::Key(key_event) = event::read()? {
                match key_event.code {
                    KeyCode::Esc => {
                        break 'gameloop;
                    }
                    KeyCode::Left => {
                        column.move_left(&pit.heap);
                    }
                    KeyCode::Right => {
                        column.move_right(&pit.heap);
                    }
                    KeyCode::Down => {
                        column.move_down(&pit.heap);
                    }
                    KeyCode::Char(' ') | KeyCode::Enter => {
                        column.cycle();
                    }
                    _ => {}
                }
            }
        }

        pit.update(&mut column, delta);
        if pit.stable() && !column.update(&pit.heap, delta) {
            column = upcoming_column;
            upcoming_column = Column::new();
        }

        // draw elements on the current frame
        pit.draw(&mut curr_frame);
        column.draw(&mut curr_frame);
        // render
        render_tx
            .send(curr_frame)
            .expect("Failed sending curr_frame to the render thread");

        if pit.topped_up() {
            // lose game
            thread::sleep(Duration::from_secs(1));
            break;
        }

        thread::sleep(fps_duration.saturating_sub(instant.elapsed()));
    }

    // Cleanup
    stdout.execute(Show)?.execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;

    Ok(())
}
