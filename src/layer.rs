use crate::{CONTROLS_HEIGHT, TOWER_WINDOW_HEIGHT, WINDOW_HEIGHT, WINDOW_WIDTH};
use crossterm::style::{self, Color, Colors};
use crossterm::{cursor, queue, terminal};
use std::fs;
use std::io::{self, Write};

pub struct Layer {
    pub style: LayerType,
    pub revenue_per_occupant: usize,
    pub occupancy: usize,
    pub max_occupancy: usize,
    pub text: String,
    //pub start_row: Option<usize>
}

impl Layer {
    pub fn new(style: LayerType, revenue_per_occupant: usize, max_occupancy: usize) -> Self {
        Self {
            style,
            revenue_per_occupant,
            occupancy: 0,
            max_occupancy,
            text: "".to_string(),
        }
    }

    pub fn revenue(&self) -> usize {
        self.occupancy * self.revenue_per_occupant
    }

    pub fn set_string(&mut self) {
        const TOWER_WIDTH: usize = 24;
        let name = match self.style {
            LayerType::Apartment => "APARTMENT COMPLEX",
            LayerType::Retail => "RETAIL STORE",
            LayerType::Food => "FOOD COURT",
        };

        let name = format!("|{name:^TOWER_WIDTH$}|\n|{:TOWER_WIDTH$}|\n", "");
        let revenue: String = format!("|       REVENUE: {:<4}    |\n", self.revenue());
        let occupancy: String = format!("|      OCCUPANCY: {:<4}   |\n", self.occupancy);
        self.text = name + &revenue + &occupancy;
    }
}

pub enum LayerType {
    // Ground,
    Apartment,
    Retail,
    Food, // Roof,
          // Ceiling
}

trait Style {
    fn colors(&self) -> Colors;
}
impl Style for LayerType {
    fn colors(&self) -> Colors {
        match self {
            // LayerType::Ground => Colors {
            //     foreground: Some(Color::White),
            //     background: Some(Color::Black),
            // },
            // LayerType::Roof => Colors {
            //     foreground: Some(Color::Cyan),
            //     background: Some(Color::Black),
            // },
            LayerType::Apartment => Colors {
                foreground: Some(Color::Green),
                background: Some(Color::Black),
            },
            LayerType::Retail => Colors {
                foreground: Some(Color::Magenta),
                background: Some(Color::Black),
            },
            LayerType::Food => Colors {
                foreground: Some(Color::Yellow),
                background: Some(Color::Black),
            },
            // LayerType::Ceiling => Colors {
            //     foreground: Some(Color::White),
            //     background: Some(Color::Black),
            // },
        }
    }
}

struct LayerText {
    text: String,
    start_row: u16,
    colors: Colors,
}
impl LayerText {
    pub fn new(text: String, num_rows: u16, start_row: u16) -> Self {
        Self {
            text: text,
            start_row: start_row,
            colors: Colors {
                foreground: Some(Color::White),
                background: Some(Color::Black),
            },
        }
    }
}

pub fn layer_draw(layers: &Vec<Layer>, mut writer: impl Write, scroll_pos: u16) -> io::Result<()> {
    

    // Set up vector of LayerText objects
    let mut layer_strings = vec![];

    // Set up Ground, add to LayerText vector
    let filepaths = vec!["src/graphics/ground.txt", "src/graphics/roof.txt"];
    let text = fs::read_to_string(filepaths[0]).expect("Was not able to read the file");
    layer_strings.push(LayerText {
        text: text,
        start_row: 1,
        colors: Colors {
            foreground: Some(Color::White),
            background: Some(Color::Black),
        },
    });

    // Set up floor layers, add to LayerText vector
    let mut row: u16 = 1;
    let iterator = layers.iter();
    let ceiling_text = "|________________________|\n";
    for layer in iterator {
        row += 4;
        layer_strings.push(LayerText {
            text: layer.text.to_string(),
            start_row: row,
            colors: layer.style.colors(),
        });
        row += 1;
        layer_strings.push(LayerText {
            text: ceiling_text.to_string(),
            start_row: row,
            colors: Colors {
                foreground: Some(Color::White),
                background: Some(Color::Black),
            },
        });
    }

    // Set up roof/cloud graphics, add to LayerText vector
    let text = fs::read_to_string(filepaths[1]).expect("Was not able to read the file");
    row += 8;
    layer_strings.push(LayerText {
        text: text,
        start_row: row,
        colors: Colors {
            foreground: Some(Color::Cyan),
            background: Some(Color::Black),
        },
    });

    queue!(
        writer,
        cursor::MoveTo(WINDOW_WIDTH, TOWER_WINDOW_HEIGHT),
        terminal::Clear(terminal::ClearType::FromCursorUp),
    )?;

    // Systematically print the relevant layers based on the scroll position
    for layer in layer_strings.iter().rev() {
        queue!(writer, style::SetColors(layer.colors))?;
        for (s, line) in (scroll_pos..).zip(layer.text.lines()) {
            if layer.start_row <= TOWER_WINDOW_HEIGHT + s && s < layer.start_row {
                queue!(
                    writer,
                    cursor::MoveTo(0, TOWER_WINDOW_HEIGHT + s - layer.start_row,),
                    style::Print(line),
                )?;
            }
        }
    }

    Ok(())
}

pub fn funds_draw(mut writer: impl Write, funds: i32, debt_timer: u32, msg:&str) -> io::Result<()> {
    // const MESSAGE_HEIGHT:u16 = T - 1;
    const INFO_MSG_WIDTH:u16 = 35;
    // Print message regarding debt collection, purchase, or other above the controls menu
    queue!(
        writer,
        cursor::MoveTo(INFO_MSG_WIDTH, TOWER_WINDOW_HEIGHT - 1),
        style::SetColors(Colors {
            foreground: Some(Color::White),
            background: Some(Color::Black),
        }),
        style::Print(msg),
    )?;

    // Print current funds and time until debt collector on the right of the tower
    let fund_string : &str = &(format!("CURRENT FUNDS: {}", funds));
    queue!(
        writer,
        cursor::MoveTo(INFO_MSG_WIDTH,0),
        style::SetColors(Colors {
            foreground: Some(Color::Green),
            background: Some(Color::Black),
        }),
        style::Print(fund_string),
    )?;
    let debt_string : &str = &(format!("TIME UNTIL DEBT COLLECTOR COMES: {}", debt_timer));
    queue!(
        writer,
        cursor::MoveTo(INFO_MSG_WIDTH,1),
        style::SetColors(Colors {
            foreground: Some(Color::Red),
            background: Some(Color::Black),
        }),
        style::Print(debt_string),
    )?;


    Ok(())

}