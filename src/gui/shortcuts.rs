use std::{collections::HashMap, fmt::Display};

use super::config_mod::KeyCommand;
use eframe::{egui::{Button, Context, Key, KeyboardShortcut, Modifiers, Ui, Window}, epaint::{Stroke, Color32}};
use std::fmt::Write as _;

pub struct ShortcutManager {
    shortcuts: HashMap<KeyCommand, KeyboardShortcut>,
    show_window: bool,
    waiting_for_input: bool,
    editing_command: KeyCommand,
    input_changed: bool,
    key_temp: Option<Key>,
    shortcut_invalid: Option<KeyCommand>,
}

impl Default for ShortcutManager {
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
            shortcuts: map,
            show_window: false,
            waiting_for_input: false,
            editing_command: KeyCommand::None,
            input_changed: false,
            key_temp: None,
            shortcut_invalid: None,
        };
    }
}

fn check_valid_shortcut(
    shortcuts: &HashMap<KeyCommand, KeyboardShortcut>,
    test_key: Key,
    test_command: KeyCommand,
) -> Option<KeyCommand> {
    for (command, shortcut) in shortcuts.iter() {
        if test_command != command.clone() && shortcut.key == test_key {
            return Some(command.clone());
        }
    }
    return None;
}

impl ShortcutManager {
    pub fn render_window(&mut self, ui: &mut Ui) {
        Window::new("Shortcuts Editor".to_string())
            .open(&mut self.show_window)
            .resize(|r| r.resizable(true))
            .show(ui.ctx(), |ui| {
                if self.waiting_for_input {
                    ui.label("Press the key you want to use as shortcut");
                    match self.key_temp {
                        Some(key) => {
                            let mut s = String::new();
                            match write!(&mut s, "{:?}", key) {
                                Ok(_) => {
                                    ui.label(s);
                                }
                                Err(_) => {}
                            }
                        }
                        None => {}
                    }
                    ui.input(|i| match i.keys_down.iter().next() {
                        Some(k) => {
                            self.key_temp = Some(k.clone());
                            self.input_changed = true;
                        }
                        None => {}
                    });
                    match &self.shortcut_invalid {
                        Some(command) => {
                            ui.add(eframe::egui::Separator::default());
                            eframe::egui::Grid::new("Invalid Shortcut").show(ui, |ui| {
                                ui.label("This shortcut is already in use for :");
                                ui.label(command.to_string());
                            });
                            ui.add(eframe::egui::Separator::default());
                        }
                        None => {}
                    }
                    ui.add(eframe::egui::Separator::default());
                    eframe::egui::Grid::new(self.waiting_for_input).show(ui, |ui| {
                        if ui.add(Button::new("Confirm")).clicked() {
                            match check_valid_shortcut(
                                &self.shortcuts,
                                self.key_temp.unwrap().clone(),
                                self.editing_command.clone(),
                            ) {
                                None => {
                                    if self.input_changed {
                                        match self.shortcuts.get_mut(&self.editing_command) {
                                            Some(s) => {
                                                s.key = self.key_temp.unwrap();
                                            }
                                            None => {}
                                        }
                                    }
                                    self.waiting_for_input = false;
                                    self.input_changed = false;
                                    self.shortcut_invalid = None;
                                }
                                Some(command) => {
                                    self.shortcut_invalid = Some(command);
                                }
                            }
                        }

                        if ui.add(Button::new("Cancel")).clicked() {
                            self.waiting_for_input = false;
                            self.input_changed = false;
                            self.key_temp = None;
                            self.shortcut_invalid = None;
                        }
                    });
                } else {
                    
                    for (command, shortcut) in self.shortcuts.iter() {

                        ui.columns(3, |columns| {
                            columns[0].label(format!("{}",command));
                            columns[1].label(format!("CTRL + {}",VirtualKey::from_key(shortcut.key.clone())));
                            if columns[2].add(eframe::egui::Button::new("Edit")).clicked() {
                                self.waiting_for_input = true;
                                self.editing_command = command.clone();
                                self.key_temp = Some(shortcut.key.clone());
                            }
                            
                        });
                        ui.add(eframe::egui::Separator::default());
/*
                        eframe::egui::Grid::new(command).show(ui, |ui| {
                            
                            ui.with_layout(eframe::egui::Layout::left_to_right(eframe::egui::Align::Center), |ui| {
                            eframe::egui::Grid::new(shortcut).show(ui, |ui| {
                                
                                ui.label(format!("{}",command));
                                ui.label(format!("CTRL + {}",VirtualKey::from_key(shortcut.key.clone())));
                                //ui.label(format!("{}",VirtualKey::from_key(shortcut.key.clone())));
                                
                            });
                            
                            
                            if ui.add(eframe::egui::Button::new("edit")).clicked() {
                                self.waiting_for_input = true;
                                self.editing_command = command.clone();
                                self.key_temp = Some(shortcut.key.clone());
                            }
                             
                        });
                        
                        });
                        ui.add(eframe::egui::Separator::default());
*/
                    }
                }
            });
    }

    pub fn show_window(&mut self) {
        return self.show_window = true;
    }

    pub fn check_shortcut(&mut self, ctx: &Context, command: &KeyCommand) -> bool {
        ctx.input_mut(|i| i.consume_shortcut(&self.shortcuts.get(command).unwrap().clone()))
    }
}

struct VirtualKey {
    key: Key,
}

impl Display for VirtualKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,"{}",self.to_string())
    }
}

impl VirtualKey {
    fn from_key(key : Key) -> Self {
        return Self {key : key};
    }
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
