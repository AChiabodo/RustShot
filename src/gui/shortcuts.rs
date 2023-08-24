use std::collections::HashMap;

use eframe::egui::{KeyboardShortcut, Modifiers, Key, Ui, Window, Context};

use super::config_mod::KeyCommand;


pub struct ShortcutManager {
    shortcuts: HashMap<KeyCommand, KeyboardShortcut>,
    show_window : bool
}

impl Default for ShortcutManager{
    fn default() -> Self {
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
        KeyCommand::Edit,
        KeyboardShortcut {
            modifiers: Modifiers::CTRL,
            key: Key::E,
        },
    );
    map.insert(
        KeyCommand::Copy,
        KeyboardShortcut {
            modifiers: Modifiers::CTRL,
            key: Key::C,
        },
    );
    return Self {
        shortcuts: map, show_window : false
    };
    }
}


impl ShortcutManager{
    pub fn render_window(&mut self,ui : &mut Ui) {
        let mut temp: HashMap<KeyCommand, KeyboardShortcut> = self.shortcuts.clone();
        let mut changed = false;
        let mut waiting_for_input = false;
        Window::new("Shortcuts Editor".to_string()).open(&mut self.show_window).resize(|r| r.resizable(true)).show(ui.ctx(), |ui|
            {
                for (command , shortcut) in temp.iter_mut() {
                    //let visual : String = VirtualKey { key: shortcut.key.clone() }.to_string();
                    ui.label(format!("{} - CTRL + {}",command,VirtualKey { key: shortcut.key.clone() }.to_string()));
                    
                    let button = ui.add(eframe::egui::Button::new("edit"));
                    
                    Window::new("Waiting for input".to_string()).open(&mut waiting_for_input).show(ui.ctx(), |ui| {
                        ui.label("Press the key you want to use as shortcut");
                        ui.label("Press ESC to cancel");
                        ui.label("Press ENTER to confirm");
                        let mut key = shortcut.key.clone();
                        ui.input(|i| {
                           println!("Input: {:?}",i.keys_down);
                           match i.keys_down.iter().next() {
                                  Some(k) => {
                                    println!("Key: {:?}",k);
                                    key = k.clone();
                                    changed = true;
                                  },
                                  None => {}  
                           } 
                        });
                    });
                    
                    if button.clicked() {
                        waiting_for_input = true;
                    }
                }                
            });
    }

    pub fn show_window(&mut self) {
        return self.show_window = true;
    }

    pub fn check_shortcut(&mut self, ctx: &Context, command : &KeyCommand) -> bool {
        ctx.input_mut(|i| {
            i.consume_shortcut(&self.shortcuts.get(command).unwrap().clone())
        })
    }

}

struct VirtualKey {
    key: Key,
}

impl VirtualKey{
    fn to_string(&self) -> String {
        match self.key {
            Key::A => "A".to_string(),
            Key::B => "B".to_string(),
            Key::C => "C".to_string(),
            Key::D => "D".to_string(),
            Key::E => "E".to_string(),
            Key::F => "F".to_string(),
            Key::G => "G".to_string(),
            Key::H => "H".to_string(),
            Key::I => "I".to_string(),
            Key::J => "J".to_string(),
            Key::K => "K".to_string(),
            Key::L => "L".to_string(),
            Key::M => "M".to_string(),
            Key::N => "N".to_string(),
            Key::O => "O".to_string(),
            Key::P => "P".to_string(),
            Key::Q => "Q".to_string(),
            Key::R => "R".to_string(),
            Key::S => "S".to_string(),
            Key::T => "T".to_string(),
            Key::U => "U".to_string(),
            Key::V => "V".to_string(),
            Key::W => "W".to_string(),
            Key::X => "X".to_string(),
            Key::Y => "Y".to_string(),
            Key::Z => "Z".to_string(),
            Key::Num0 => "0".to_string(),
            Key::Num1 => "1".to_string(),
            Key::Num2 => "2".to_string(),
            Key::Num3 => "3".to_string(),
            Key::Num4 => "4".to_string(),
            Key::Num5 => "5".to_string(),
            Key::Num6 => "6".to_string(),
            Key::Num7 => "7".to_string(),
            Key::Num8 => "8".to_string(),
            Key::Num9 => "9".to_string(),
            Key::Space => "Space".to_string(),
            Key::Tab => "Tab".to_string(),
            Key::Backspace => "Backspace".to_string(),
            Key::Delete => "Delete".to_string(),
            Key::Insert => "Insert".to_string(),
            Key::ArrowDown => "ArrowDown".to_string(),
            Key::ArrowLeft => "ArrowLeft".to_string(),
            Key::ArrowRight => "ArrowRight".to_string(),
            Key::ArrowUp => "ArrowUp".to_string(),
            Key::Home => "Home".to_string(),
            Key::End => "End".to_string(),
            Key::PageUp => "PageUp".to_string(),
            Key::PageDown => "PageDown".to_string(),
            Key::Escape => "Escape".to_string(),
            Key::Enter => "Enter".to_string(),
            Key::F1 => "F1".to_string(),
            Key::F2 => "F2".to_string(),
            Key::F3 => "F3".to_string(),
            Key::F4 => "F4".to_string(),
            Key::F5 => "F5".to_string(),
            Key::F6 => "F6".to_string(),
            Key::F7 => "F7".to_string(),
            Key::F8 => "F8".to_string(),
            Key::F9 => "F9".to_string(),
            Key::F10 => "F10".to_string(),
            Key::F11 => "F11".to_string(),
            Key::F12 => "F12".to_string(),
            Key::F13 => "F13".to_string(),
            Key::F14 => "F14".to_string(),
            Key::F15 => "F15".to_string(),
            Key::F16 => "F16".to_string(),
            Key::F17 => "F17".to_string(),
            Key::F18 => "F18".to_string(),
            Key::F19 => "F19".to_string(),
            Key::F20 => "F20".to_string(),
            Key::Minus => "Minus".to_string(),
            Key::PlusEquals => "PlusEquals".to_string(),
        }
    }
}