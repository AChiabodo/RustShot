use std::collections::HashMap;
use eframe::egui::{Key, KeyboardShortcut, Modifiers};
use egui_extras::RetainedImage;

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum KeyCommand {
    SaveScreenshot,
    TakeScreenshot,
    Crop,
    Edit,
    None,
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

pub fn load_shortcuts() -> HashMap<KeyCommand, KeyboardShortcut> {
    let mut map = HashMap::new();
    map.insert(
        KeyCommand::SaveScreenshot,
        KeyboardShortcut {
            modifiers: Modifiers::CTRL,
            key: Key::S,
        },
    );
    map.insert(
        KeyCommand::TakeScreenshot,
        KeyboardShortcut {
            modifiers: Modifiers::CTRL,
            key: Key::T,
        },
    );
    map.insert(
        KeyCommand::Crop,
        KeyboardShortcut {
            modifiers: Modifiers::CTRL,
            key: Key::C,
        },
    );
    map.insert(
        KeyCommand::Edit,
        KeyboardShortcut {
            modifiers: Modifiers::CTRL,
            key: Key::P,
        },
    );
    return map;
}