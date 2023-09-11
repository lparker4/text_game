use crate::{GameState, CONTROLS_HEIGHT, WINDOW_HEIGHT};
use std::collections::HashMap;

use crossterm::event::{KeyCode, KeyModifiers};
use crossterm::style::{self, style, Stylize};
use crossterm::{cursor, queue, terminal};
use std::io::{self, Write};

pub type CommandFn = fn(&mut GameState) -> io::Result<()>;

#[derive(Copy, Clone)]
pub struct Command {
    op: CommandFn,
    is_triggered_on_repeat: bool,
}

impl Command {
    pub fn new(op: CommandFn) -> Self {
        Command {
            op,
            is_triggered_on_repeat: false,
        }
    }

    pub fn with_triggered_on_repeat(op: CommandFn, is_triggered_on_repeat: bool) -> Self {
        Command {
            op,
            is_triggered_on_repeat,
        }
    }

    pub fn is_triggered_on_repeat(&self) -> bool {
        self.is_triggered_on_repeat
    }

    pub fn execute(&self, gs: &mut GameState) -> io::Result<()> {
        (self.op)(gs)
    }
}

pub struct CommandPool {
    commands: HashMap<(KeyCode, KeyModifiers), Command>,
    descriptions: Vec<String>,
}

impl CommandPool {
    pub fn get_command(&self, key_code: KeyCode, key_modifiers: KeyModifiers) -> Option<Command> {
        self.commands.get(&(key_code, key_modifiers)).copied()
    }

    pub fn draw(&self, mut writer: impl Write) -> io::Result<()> {
        const COL_WIDTH: u16 = 16;

        queue!(
            writer,
            cursor::MoveTo(0, WINDOW_HEIGHT - CONTROLS_HEIGHT),
            terminal::Clear(terminal::ClearType::FromCursorDown),
        )?;
        for (desc, (col, row)) in self.descriptions.iter().zip(iter_cols_rows()) {
            queue!(
                writer,
                cursor::MoveTo(
                    1 + col * COL_WIDTH,
                    WINDOW_HEIGHT - CONTROLS_HEIGHT + 1 + 2 * row
                ),
                style::Print(desc),
            )?;
        }
        Ok(())
    }
}

pub struct CommandPoolBuilder {
    pool: CommandPool,
}

impl CommandPoolBuilder {
    pub fn new() -> Self {
        CommandPoolBuilder {
            pool: CommandPool {
                commands: HashMap::new(),
                descriptions: Vec::new(),
            },
        }
    }

    pub fn on_letter_press(
        mut self,
        key: char,
        description: impl AsRef<str>,
        command: Command,
    ) -> Self {
        assert!(key.is_ascii_lowercase());
        self.pool
            .commands
            .insert((KeyCode::Char(key), KeyModifiers::empty()), command);
        self.pool.descriptions.push(format!(
            "{:>5}: {}",
            style(key).green(),
            description.as_ref(),
        ));
        self
    }

    pub fn build(self) -> CommandPool {
        self.pool
    }
}

fn iter_cols_rows() -> impl Iterator<Item = (u16, u16)> {
    const TOTAL_ROWS: u16 = (CONTROLS_HEIGHT - 1) / 2;
    (0..).flat_map(|c| std::iter::repeat(c).zip(0..TOTAL_ROWS))
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub struct CommandPoolId(usize);

pub struct CommandPoolArray {
    pools: Vec<CommandPool>,
    id: CommandPoolId,
}

impl CommandPoolArray {
    pub fn id(&self) -> CommandPoolId {
        self.id
    }

    pub fn set_id(&mut self, id: CommandPoolId) {
        assert!(id.0 < self.pools.len());
        self.id = id;
    }

    pub fn cur(&self) -> &CommandPool {
        &self.pools[self.id.0]
    }
}

pub struct CommandPoolArrayBuilder {
    pools: Vec<CommandPool>,
}

impl CommandPoolArrayBuilder {
    pub fn new() -> Self {
        CommandPoolArrayBuilder { pools: Vec::new() }
    }

    pub fn add_pool(&mut self, pool: CommandPool) -> CommandPoolId {
        self.pools.push(pool);
        CommandPoolId(self.pools.len() - 1)
    }

    pub fn with_initial_pool(self, id: CommandPoolId) -> CommandPoolArray {
        assert!(id.0 < self.pools.len());
        CommandPoolArray {
            pools: self.pools,
            id,
        }
    }
}
