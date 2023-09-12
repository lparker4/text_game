use crate::command::*;
use crate::layer::*;

use std::io;

pub struct GameState {
    pub stdout: io::StdoutLock<'static>,
    pub running: bool,
    pub command_pool_array: CommandPoolArray,

    pub command_pool_a_id: CommandPoolId,
    pub command_pool_b_id: CommandPoolId,

    pub layers: Vec<Layer>,

}

impl GameState {
    pub fn command_pool(&self) -> &CommandPool {
        self.command_pool_array.cur()
    }

    pub fn draw_command_pool(&mut self) -> io::Result<()> {
        let pool = self.command_pool_array.cur();
        pool.draw(&mut self.stdout)
    }

    pub fn draw_tower(&mut self) -> io::Result<()> {
        layer_draw(&self.layers, &mut self.stdout)
    }
}

fn noop(_gs: &mut GameState) -> io::Result<()> {
    Ok(())
}

pub fn init_game_state(stdout: io::StdoutLock<'static>) -> GameState {
    let noop_command = Command::new(noop);

    let mut pool_array_builder = CommandPoolArrayBuilder::new();

    let command_pool_a_id = pool_array_builder.add_pool(
        CommandPoolBuilder::new()
            .on_letter_press('w', "loop up", noop_command)
            .on_letter_press('a', "loop left", noop_command)
            .on_letter_press('s', "look down", noop_command)
            .on_letter_press('d', "foo right", noop_command)
            .on_letter_press('z', "foo", noop_command)
            .on_letter_press('x', "bar", noop_command)
            .on_letter_press('c', "baz", noop_command)
            .on_letter_press('v', "qux", noop_command)
            .on_letter_press('u', "ascend", noop_command)
            .on_letter_press('h', "shift l", noop_command)
            .on_letter_press('m', "descend", noop_command)
            .on_letter_press('k', "shift r", noop_command)
            .build(),
    );

    let command_pool_b_id = pool_array_builder.add_pool(
        CommandPoolBuilder::new()
            .on_letter_press('w', "wah", noop_command)
            .on_letter_press('a', "wah", noop_command)
            .on_letter_press('s', "wee", noop_command)
            .on_letter_press('d', "wah", noop_command)
            .on_letter_press('z', "hoho", noop_command)
            .on_letter_press('x', "hoh...", noop_command)
            .build(),
    );
    let mut testLayer : Layer = Layer{style:LayerType::Apartment, occupancy:0, revenue:0, text:"".to_string()};
    testLayer.set_string();

    let layers: Vec<Layer> = vec![testLayer];

    GameState {
        running: true,
        stdout,
        command_pool_array: pool_array_builder.with_initial_pool(command_pool_a_id),
        command_pool_a_id,
        command_pool_b_id,
        layers,
    }
}
