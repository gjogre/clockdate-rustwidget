mod font;
use chrono::Local;
use crossterm::{
    event::{self, EnableMouseCapture, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    prelude::Alignment,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Paragraph},
};

use std::{
    error::Error,
    io,
    time::{Duration, Instant},
};

fn main() -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let res = run_app(&mut terminal);

    disable_raw_mode()?;
    // execute!(
    //     terminal.backend_mut(),
    //     LeaveAlternateScreen,
    //     DisableMouseCapture
    // )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        eprintln!("Error: {:?}", err);
    }

    Ok(())
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
) -> Result<(), Box<dyn Error>> {
    let tick_rate = Duration::from_millis(250);
    let mut last_tick = Instant::now();

    let (font_time, font_date) =
        font::load_embedded_figlet_fonts().expect("Failed to load embedded font");
    loop {
        terminal.draw(|f| ui(f, &font_time, &font_date))?;

        let timeout = tick_rate.saturating_sub(last_tick.elapsed());

        if event::poll(timeout)? {
            if let event::Event::Key(key) = event::read()? {
                if key.code == KeyCode::Char('q') {
                    break;
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();
        }
    }

    Ok(())
}

fn ui(f: &mut Frame, font_time: &figlet_rs::FIGfont, font_date: &figlet_rs::FIGfont) {
    let size = f.size();
    let now = Local::now();

    let time_str = now.format("%H:%M").to_string();
    let date_str = now.format("%d.%m.%Y").to_string();

    let figlet_time_text = font::render_figlet_text(font_time, &time_str);
    let figlet_date_text = font::render_figlet_text(font_date, &date_str);

    let time_style = Style {
        fg: Some(Color::Blue),
        bg: Some(Color::default()),
        underline_color: Some(Color::default()),
        add_modifier: Modifier::empty(),
        sub_modifier: Modifier::empty(),
    };

    let date_style = Style {
        fg: Some(Color::Blue),
        bg: Some(Color::default()),
        underline_color: Some(Color::default()),
        add_modifier: Modifier::DIM,
        sub_modifier: Modifier::empty(),
    };

    let time_paragraph = Paragraph::new(figlet_time_text.to_string())
        .block(Block::default().borders(Borders::LEFT | Borders::TOP | Borders::RIGHT))
        .style(time_style)
        .alignment(Alignment::Center);

    let date_paragraph = Paragraph::new(figlet_date_text.to_string())
        .block(Block::default().borders(Borders::LEFT | Borders::BOTTOM | Borders::RIGHT))
        .style(date_style)
        .alignment(Alignment::Center);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(5), Constraint::Percentage(5)].as_ref())
        .split(size);

    f.render_widget(time_paragraph, chunks[0]);

    f.render_widget(date_paragraph, chunks[1]);
}
