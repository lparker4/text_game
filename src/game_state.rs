use crate::command::*;
use crate::layer::*;

use std::io;

pub struct GameState {
    pub stdout: io::StdoutLock<'static>,
    pub running: bool,
    pub command_pool_array: CommandPoolArray,

    pub command_pool_main_id: CommandPoolId,
    pub command_pool_build_id: CommandPoolId,

    pub layers: Vec<Layer>,
    pub scroll_pos: u16,
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
        layer_draw(&self.layers, &mut self.stdout, self.scroll_pos)
    }

    pub fn enter_menu(&mut self, command_pool_id: CommandPoolId) -> io::Result<()> {
        self.command_pool_array.set_id(command_pool_id);
        self.draw_command_pool()
    }

    pub fn add_layer(&mut self, style: LayerType) {
        let mut new_layer = Layer::new(style, 0, 0);
        new_layer.set_string();
        self.layers.push(new_layer);
    }
}

fn noop(_gs: &mut GameState) -> io::Result<()> {
    Ok(())
}

pub fn init_game_state(stdout: io::StdoutLock<'static>) -> GameState {
    let mut pool_array_builder = CommandPoolArrayBuilder::new();

    let command_pool_main_id = pool_array_builder.add_pool(
        CommandPoolBuilder::new()
            .on_letter_press('w', "Scroll up", Command::new(|gs| {
                gs.scroll_pos += 1;
                gs.draw_tower()
            }))
            .on_letter_press('s', "Scroll down", Command::new(|gs| {
                gs.scroll_pos = gs.scroll_pos.saturating_sub(1);
                gs.draw_tower()
            }))
            .on_letter_press('b', "Build", Command::new(|gs| {
                gs.enter_menu(gs.command_pool_build_id)
            }))
            .build(),
    );

    let command_pool_build_id = pool_array_builder.add_pool(
        CommandPoolBuilder::new()
            .on_letter_press('f', "Food court", Command::new(|gs| {
                gs.add_layer(LayerType::Food);
                gs.draw_tower()?;
                gs.enter_menu(gs.command_pool_main_id)
            }))
            .on_letter_press('a', "Apartments", Command::new(|gs| {
                gs.add_layer(LayerType::Apartment);
                gs.draw_tower()?;
                gs.enter_menu(gs.command_pool_main_id)
            }))
            .on_letter_press('r', "Retail", Command::new(|gs| {
                gs.add_layer(LayerType::Retail);
                gs.draw_tower()?;
                gs.enter_menu(gs.command_pool_main_id)
            }))
            .on_letter_press('x', "Cancel", Command::new(|gs| {
                gs.enter_menu(gs.command_pool_main_id)
            }))
            .build()
    );

    let mut test_layer_1: Layer = Layer {
        style: LayerType::Apartment,
        occupancy: 0,
        revenue: 0,
        text: "".to_string(),
    };
    let mut test_layer_2: Layer = Layer {
        style: LayerType::Food,
        occupancy: 24,
        revenue: 300,
        text: "".to_string(),
    };

    test_layer_1.set_string();
    test_layer_2.set_string();

    let layers: Vec<Layer> = vec![test_layer_1, test_layer_2];

    GameState {
        running: true,
        stdout,
        command_pool_array: pool_array_builder.with_initial_pool(command_pool_main_id),
        command_pool_main_id,
        command_pool_build_id,
        layers,
        scroll_pos: 0,
    }
}
