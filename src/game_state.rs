use crate::command::*;
use crate::layer::*;
use rand::prelude::*;
use std::io;

pub struct GameState {
    pub stdout: io::StdoutLock<'static>,
    pub running: bool,
    pub restarting: bool,
    pub command_pool_array: CommandPoolArray,

    pub command_pool_main_id: CommandPoolId,
    pub command_pool_build_id: CommandPoolId,
    pub command_pool_game_over_id: CommandPoolId,

    pub layers: Vec<Layer>,
    pub scroll_pos: u16,

    pub funds: i32,
    pub debt_collection_timer: u32,
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

    pub fn draw_funds(&mut self, charges: i32, debits: i32) -> io::Result<()> {
        let mut msg: String = "".to_string();
        if charges > 0 {
            if debits > 0{
                msg = format!("You were charged ${}, and made ${}", charges, debits);
            }
            msg = format!("You were charged ${}", charges);
        }else if debits > 0 {
            msg = format!("You made ${}", debits);
        }
        if self.debt_collection_timer == 0 {
            if self.funds >= 0{
                msg = "YOU PASSED THE DEBT COLLECTOR'S INSPECTION!".to_string();
                self.debt_collection_timer = 50;
            } else {
                msg = "YOU DID NOT HAVE FUNDS TO PAY THE DEBT COLLECTOR. GAME OVER.".to_string();
                self.enter_menu(self.command_pool_game_over_id)?;
            }
        }
        self.funds += debits;
        self.funds -= charges;
        funds_draw(&mut self.stdout, self.funds, self.debt_collection_timer, &msg)
    }

    pub fn enter_menu(&mut self, command_pool_id: CommandPoolId) -> io::Result<()> {
        self.command_pool_array.set_id(command_pool_id);
        self.draw_command_pool()
    }

    pub fn add_layer(&mut self, style: LayerType) {
        let (max_occupancy, revenue_per_occupant) = match style {
            LayerType::Apartment => (20, 17),
            LayerType::Food => (30, 7),
            LayerType::Retail => (10, 12),
        };
        let mut new_layer = Layer::new(style, revenue_per_occupant, max_occupancy);
        new_layer.set_string();
        self.layers.push(new_layer);
    }

    pub fn update_occupancies(&mut self) {
        let total_occupants: usize = self.layers.iter().map(|x| x.occupancy).sum();
        let move_in_thresh = 1.0 - 0.4 * std::f64::consts::E.powf(-0.01 * total_occupants as f64);
        let move_out_thresh = 0.3 * std::f64::consts::E.powf(-0.005 * total_occupants as f64);
        for layer in &mut self.layers {
            let n = thread_rng().gen_range(0.0..1.0);
            if n < move_out_thresh {
                layer.occupancy = layer.occupancy.saturating_sub(1);
            } else if n < move_in_thresh {
                layer.occupancy = layer.max_occupancy.min(layer.occupancy + 1);
            }
            layer.set_string();
        }
    }
}

pub fn init_game_state(stdout: io::StdoutLock<'static>) -> GameState {
    let mut pool_array_builder = CommandPoolArrayBuilder::new();

    let command_pool_main_id = pool_array_builder.add_pool(
        CommandPoolBuilder::new()
            .on_letter_press(
                'w',
                "Scroll up",
                Command::new(|gs| {
                    gs.scroll_pos += 1;
                    gs.draw_tower()?;
                    gs.draw_funds(0, 0)
                }),
            )
            .on_letter_press(
                's',
                "Scroll down",
                Command::new(|gs| {
                    gs.scroll_pos = gs.scroll_pos.saturating_sub(1);
                    gs.draw_tower()?;
                    gs.draw_funds(0, 0)
                }),
            )
            .on_letter_press(
                'b',
                "Build",
                Command::new(|gs| gs.enter_menu(gs.command_pool_build_id)),
            )
            .build(),
    );

    let command_pool_build_id = pool_array_builder.add_pool(
        CommandPoolBuilder::new()
            .on_letter_press('f', "Food court- $10000", Command::new(|gs| {
                gs.add_layer(LayerType::Food);
                gs.draw_tower()?;
                gs.draw_funds(10000, 0)?;
                gs.enter_menu(gs.command_pool_main_id)
            }))
            .on_letter_press('a', "Apartments- $12000", Command::new(|gs| {
                gs.add_layer(LayerType::Apartment);
                gs.draw_tower()?;
                gs.draw_funds(12000, 0)?;
                gs.enter_menu(gs.command_pool_main_id)
            }))
            .on_letter_press('r', "Retail- $8000", Command::new(|gs| {
                gs.add_layer(LayerType::Retail);
                gs.draw_tower()?;
                gs.draw_funds(8000, 0)?;
                gs.enter_menu(gs.command_pool_main_id)
            }))
            .on_letter_press('x', "Cancel", Command::new(|gs| {
                gs.enter_menu(gs.command_pool_main_id)
            }))
            .build()
    );

    let command_pool_game_over_id = pool_array_builder.add_pool(
        CommandPoolBuilder::new()
            .on_letter_press('r', "Retry", Command::new(|gs| {
                gs.restarting = true;
                Ok(())
            }))
            .on_letter_press('x', "Exit", Command::new(|gs| {
                gs.running = false;
                Ok(())
            }))
            .build()
    );

    GameState {
        running: true,
        restarting: false,
        stdout,
        command_pool_array: pool_array_builder.with_initial_pool(command_pool_main_id),
        command_pool_main_id,
        command_pool_build_id,
        command_pool_game_over_id,
        layers: vec![],
        scroll_pos: 0,
        funds: 10_000,
        debt_collection_timer: 50,
    }
}
