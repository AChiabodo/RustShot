use std::{collections::HashMap, fmt::Display};
use eframe::egui::{Key, KeyboardShortcut, Modifiers};
use egui_extras::RetainedImage;

use super::shortcuts::{self, ShortcutManager};

#[derive(Debug,PartialEq, Eq, PartialOrd, Ord, Hash,Clone)]
pub enum KeyCommand {
    SaveScreenshot,
    TakeScreenshot,
    Edit,
    Copy,
    None,
}

impl KeyCommand {
    pub fn to_string(&self) -> String {
        match self {
            KeyCommand::Edit => {
                return "Edit".to_string();
            },
            KeyCommand::SaveScreenshot => {
                return "Save Screenshot".to_string();
            },
            KeyCommand::TakeScreenshot => {
                return "Take Screenshot".to_string();
            },
            KeyCommand::None => {
                return "None".to_string();
            },
            KeyCommand::Copy => {
                return "Copy".to_string();
            },

        }
    }
}

impl Display for KeyCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,"{}",self.to_string())
    }
}

/// Load in the application state the svg icons as RetainedImage, and also the correspondence between the backend name of the icon and its tooltip.
pub fn load_icons() -> (HashMap<String, Result<RetainedImage, String>>, HashMap<String, String>, ) {
    let mut icons_map = HashMap::new();
    let mut tooltips_map = HashMap::new();
    icons_map.insert(
        "pencil-fill".to_string(),
        RetainedImage::from_svg_bytes(
            "pencil-fill",
            include_bytes!("../../resources/pencil-fill.svg"),
        ),
    );
    tooltips_map.insert("pencil-fill".to_string(), "Pencil".to_string());
    icons_map.insert(
        "square-fill".to_string(),
        RetainedImage::from_svg_bytes(
            "square-fill",
            include_bytes!("../../resources/square-fill.svg"),
        ),
    );
    tooltips_map.insert("square-fill".to_string(), "Filled Rectangle".to_string());
    icons_map.insert(
        "square".to_string(),
        RetainedImage::from_svg_bytes("square", include_bytes!("../../resources/square.svg")),
    );
    tooltips_map.insert("square".to_string(), "Hollow Rectangle".to_string());
    icons_map.insert(
        "circle-fill".to_string(),
        RetainedImage::from_svg_bytes(
            "circle-fill",
            include_bytes!("../../resources/circle-fill.svg"),
        ),
    );
    tooltips_map.insert("circle-fill".to_string(), "Filled Circle".to_string());
    icons_map.insert(
        "circle".to_string(),
        RetainedImage::from_svg_bytes("circle", include_bytes!("../../resources/circle.svg")),
    );
    tooltips_map.insert("circle".to_string(), "Hollow Circle".to_string());
    icons_map.insert(
        "arrow-up-right".to_string(),
        RetainedImage::from_svg_bytes(
            "arrow-up-right",
            include_bytes!("../../resources/arrow-up-right.svg"),
        ),
    );
    tooltips_map.insert("arrow-up-right".to_string(), "Arrow".to_string());
    icons_map.insert(
        "eraser-fill".to_string(),
        RetainedImage::from_svg_bytes(
            "eraser-fill",
            include_bytes!("../../resources/eraser-fill.svg"),
        ),
    );
    tooltips_map.insert("eraser-fill".to_string(), "Eraser".to_string());
    icons_map.insert(
        "x-octagon".to_string(),
        RetainedImage::from_svg_bytes("x-octagon", include_bytes!("../../resources/x-octagon.svg")),
    );
    tooltips_map.insert("x-octagon".to_string(), "Stop using this tool".to_string());
    icons_map.insert(
        "highlighter-solid".to_string(),
        RetainedImage::from_svg_bytes("highlighter-solid", include_bytes!("../../resources/highlighter-solid.svg")),
    );
    tooltips_map.insert("highlighter-solid".to_string(), "Highlight".to_string());
    icons_map.insert(
        "crop".to_string(),
        RetainedImage::from_svg_bytes("crop", include_bytes!("../../resources/crop.svg")),
    );
    tooltips_map.insert("crop".to_string(), "Crop".to_string());
    icons_map.insert(
        "pentagon".to_string(),
        RetainedImage::from_svg_bytes("pentagon", include_bytes!("../../resources/pentagon.svg")),
    );
    tooltips_map.insert("pentagon".to_string(), "Shape".to_string());
    icons_map.insert(
        "arrow-90deg-left".to_string(),
        RetainedImage::from_svg_bytes("arrow-90deg-left", include_bytes!("../../resources/arrow-90deg-left.svg")),
    );
    tooltips_map.insert("arrow-90deg-left".to_string(), "Undo last action".to_string());
    icons_map.insert(
        "arrow-90deg-right".to_string(),
        RetainedImage::from_svg_bytes("arrow-90deg-right", include_bytes!("../../resources/arrow-90deg-right.svg")),
    );
    tooltips_map.insert("arrow-90deg-right".to_string(), "Redo last action".to_string());
    icons_map.insert(
        "clipboard".to_string(),
        RetainedImage::from_svg_bytes(
            "clipboard",
            include_bytes!("../../resources/clipboard.svg"),
        ),
    );
    tooltips_map.insert("clipboard".to_string(), "Copy image to clipboard".to_string());
    return (icons_map, tooltips_map);
}