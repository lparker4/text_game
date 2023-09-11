use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use crossterm::{cursor, execute, terminal};
use futures::prelude::*;
use std::io;
use std::pin::pin;
use std::time::Duration;

mod command;
mod game_state;

use game_state::GameState;

pub const WINDOW_WIDTH: u16 = 120;
pub const WINDOW_HEIGHT: u16 = 30;

pub const CONTROLS_HEIGHT: u16 = 11;


#[tokio::main(flavor = "current_thread")]
async fn main() -> io::Result<()> {
    // Calling supports_ansi() on Windows may actually *cause* the terminal
    // to begin supporting ansi escape codes, so this check serves 2 purposes.
    #[cfg(windows)]
    if !crossterm::ansi_support::supports_ansi() {
        println!("Your terminal does not support ansi escape codes!");
        return Ok(())
    }
    
    let mut stdout = io::stdout().lock();
    let (original_w, original_h) = terminal::size()?;
    terminal::enable_raw_mode()?;

    execute!(
        stdout,
        terminal::SetSize(WINDOW_WIDTH, WINDOW_HEIGHT),
        terminal::EnterAlternateScreen,
        terminal::Clear(terminal::ClearType::All),
        terminal::DisableLineWrap,
        cursor::Hide,
        terminal::BeginSynchronizedUpdate,
    )?;

    let mut gs = game_state::init_game_state(stdout);

    let mut event_stream = event::EventStream::new();
    let mut heartbeat = tokio::time::interval(Duration::from_secs(1));
    'main_loop: while gs.running {
        let mut event_fut = event_stream.next().fuse();
        let mut heartbeat_fut = pin!(heartbeat.tick().fuse());
        futures::select! {
            event = event_fut => match event {
                Some(Ok(Event::Key(ke))) => handle_key_event(&mut gs, ke)?,
                None => break 'main_loop,
                Some(Err(e)) => return Err(e),
                _ => {}
            },
            _ = heartbeat_fut => handle_time_tick(&mut gs)?,
        };
        execute!(
            gs.stdout,
            terminal::EndSynchronizedUpdate,
            terminal::BeginSynchronizedUpdate,
        )?;
    }

    execute!(
        gs.stdout,
        terminal::LeaveAlternateScreen,
        cursor::Show,
        terminal::SetSize(original_w, original_h)
    )?;

    Ok(())
}

fn handle_key_event(gs: &mut GameState, ke: KeyEvent) -> io::Result<()> {
    match ke {
        KeyEvent {
            code: KeyCode::Char('c'),
            modifiers,
            kind: KeyEventKind::Press,
            ..
        } if modifiers == KeyModifiers::CONTROL => {
            gs.running = false;
            return Ok(());
        }
        _ => {}
    }

    let pool = gs.command_pool();
    let Some(command) = pool.get_command(ke.code, ke.modifiers) else {
        return Ok(());
    };
    match ke.kind {
        KeyEventKind::Press => {
            command.execute(gs)?;
        }
        KeyEventKind::Repeat => {
            if command.is_triggered_on_repeat() {
                command.execute(gs)?;
            }
        }
        _ => {}
    }

    Ok(())
}

fn handle_time_tick(gs: &mut GameState) -> io::Result<()> {
    gs.draw_command_pool()?;

    if gs.command_pool_array.id() == gs.command_pool_a_id {
        gs.command_pool_array.set_id(gs.command_pool_b_id);
    } else {
        gs.command_pool_array.set_id(gs.command_pool_a_id);
    }

    Ok(())
}
