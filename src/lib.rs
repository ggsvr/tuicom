pub mod app;
pub mod args;
pub mod dummy;
pub mod screen;
pub mod ui;

use app::App;
use args::Args;
use std::io;
use std::time::Duration;

use ratatui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("invalid serial port: {0}")]
    InvalidPort(#[from] serialport::Error),
    #[error("invalid baud rate: {0}")]
    InvalidBaudRate(u16),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}
type Result<T> = std::result::Result<T, Error>;

/// Application entry point
pub fn run_app() -> Result<()> {
    let args: Args = argh::from_env();

    let port = if args.port == "dummy" {
        Box::new(dummy::DummySerial::new(args.baud))
    } else {
        serialport::new(&args.port, args.baud)
            .timeout(Duration::from_millis(500))
            .open()?
    };
    let mut terminal = start_tui()?;
    // little trick to replace `try` block
    let res = (|| {
        let mut app = App::new(port);

        loop {
            let ev = if event::poll(Duration::from_millis(1000 / 60))? {
                Some(event::read()?)
            } else {
                None
            };
            if app.update(ev)?.exit() {
                break;
            }
            terminal.draw(|f| ui::draw(f, &mut app))?;
        }
        Ok(())
    })();
    leave_tui(terminal)?;
    res
}

fn start_tui() -> Result<Terminal<CrosstermBackend<io::Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

fn leave_tui<B: Backend + std::io::Write>(mut terminal: Terminal<B>) -> Result<()> {
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    Ok(())
}
