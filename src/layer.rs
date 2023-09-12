use crate::{GameState, CONTROLS_HEIGHT, WINDOW_HEIGHT};
use crossterm::{cursor, queue, terminal};
use crossterm::style::{self, Print, SetColors, style, Stylize};

use crossterm::style::{Color, Colors};
use core::num;
use std::fs;

use std::io::{self, Write};
pub struct Layer{
    pub style:LayerType, 
    pub revenue: usize, 
    pub occupancy: usize,
    pub text: String,
    //pub start_row: Option<usize>
}
impl Layer{
    pub fn new(style:LayerType, revenue:usize, occupancy:usize) -> Self{
        Self{style:style, revenue:revenue, occupancy:occupancy, text:"".to_string()}
    }
    pub fn set_string(&mut self){
        const TOWER_WIDTH:u32 = 24;
        let mut name :String = "".to_string();
        match self.style {
            LayerType::Apartment => {
                name += "|   APARTMENT COMPLEX    |\n|                        |\n";
            },
            LayerType::Retail => {
                name += "|      RETAIL STORE      |\n|                        |\n";
            },
            LayerType::Food => {
                name += "|       FOOD COURT       |\n|                        |\n";
            },
        }
        let revenue: String= format!("|       REVENUE: {}       |\n", self.revenue);
        let occupancy: String= format!("|      OCCUPANCY: {}      |\n", self.occupancy);
        let mut text :String = "".to_string();
        text += &name;
        text += &revenue;
        text += &occupancy;
        self.text = text;
    }
}

pub enum LayerType {
    // Ground,
    Apartment,
    Retail,
    Food
    // Roof,
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

struct LayerText{
    text:String,
    num_rows:u16,
    start_row:u16,
    colors:Colors,
}
impl LayerText{
    pub fn new(text:String, num_rows:u16, start_row:u16) -> Self{
        Self{text:text, num_rows:num_rows, start_row:start_row, colors:Colors {
            foreground: Some(Color::White),
            background: Some(Color::Black),
            }}
    }
    
}

pub fn layer_draw(layers:&Vec<Layer>, mut writer: impl Write) -> std::io::Result<()> {
    // Determine where to get this number from elsewhere
    let scroll_pos: u16 = 0;
    const TOWER_WINDOW_HEIGHT : u16 = 25;

    // Set up vector of LayerText objects
    let mut layer_strings = vec![];

    // Set up Ground, add to LayerText vector
    let filepaths = vec!["C://Users//laure//OneDrive//Documents//CS181//text_game//src//graphics//ground.txt",
                                    "C://Users//laure//OneDrive//Documents//CS181//text_game//src//graphics//roof.txt"];
    let text = fs::read_to_string(filepaths[0])
        .expect("Was not able to read the file");
    layer_strings.push(LayerText{text:text, 
                                num_rows:1, 
                                start_row:0, 
                                colors:Colors {
                                    foreground: Some(Color::White),
                                    background: Some(Color::Black),
                                    }});

    // Set up floor layers, add to LayerText vector
    let mut row : u16 = 1;
    let iterator= layers.iter();
    let ceiling_text = "|_______________________|\n";
    for layer in iterator {
        row += 4;
        layer_strings.push(LayerText{text: layer.text.to_string(), 
                                    num_rows:4, 
                                    start_row:row, 
                                    colors:layer.style.colors()});
        row += 1;
        layer_strings.push(LayerText{text:ceiling_text.to_string(), 
                                    num_rows:1, 
                                    start_row:row, 
                                    colors:Colors {
                                        foreground: Some(Color::White),
                                        background: Some(Color::Black),
                                        }});
    }

    // Set up roof/cloud graphics, add to LayerText vector
    let text = fs::read_to_string(filepaths[1])
    .expect("Was not able to read the file");
    layer_strings.push(LayerText{text:text, num_rows:7, start_row:row, colors:Colors {
        foreground: Some(Color::Cyan),
        background: Some(Color::Black),
        }});
    
    // Systematically print the relevant layers based on the scroll position
    let iterator= layer_strings.iter().rev();
    for layer in iterator{
        if (layer.start_row >= scroll_pos) && (layer.start_row < scroll_pos+TOWER_WINDOW_HEIGHT){

            queue!(
                writer,
                cursor::MoveTo(
                    0,
                    WINDOW_HEIGHT - 17 - layer.start_row,
                ),
                style::SetColors(layer.colors),
                style::Print(layer.text.to_string()),
            )?;
        }
    }

    Ok(())

}
