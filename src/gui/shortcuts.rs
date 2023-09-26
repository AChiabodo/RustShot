use super::config_mod::KeyCommand;
use eframe::egui::{Button, Context, Key, KeyboardShortcut, Modifiers, Ui, Window, ComboBox};
use global_hotkey::hotkey::{HotKey, Code};
use serde::{Deserialize, Serialize};
use std::fmt::Write as _;
use std::path::PathBuf;
use std::{collections::HashMap, fmt::Display, fs};
use rfd::FileDialog;


#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SaveHotKeys {
    modifier: global_hotkey::hotkey::Modifiers,
    key: Code,
}

impl Default for SaveHotKeys {
    fn default() -> Self {
        SaveHotKeys {
            modifier: global_hotkey::hotkey::Modifiers::ALT,
            key: Code::KeyS,
        }
    }
}

impl Into<VirtualKey> for SaveHotKeys {
    fn into(self) -> VirtualKey {
        VirtualKey::from_hotkey(self.key)
    }
}

impl Display for SaveHotKeys {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl Into<VirtualShortcut> for SaveHotKeys {
    fn into(self) -> VirtualShortcut {
        VirtualShortcut::new(Modifiers::ALT, VirtualKey::from_hotkey(self.key).into())
    }
}

impl SaveHotKeys {
    pub fn new() -> Self {
        let mut res = SaveHotKeys::default();
        return match res.read_from_disk() {
            Ok(_) => { res }
            Err(_) => {
                res.write_to_disk().unwrap();
                res
            }
        };
    }
    fn to_string(&self) -> String {
        format!("{:?} - {}", self.modifier, self.key.to_string())
    }
    fn write_to_disk(&self) -> std::io::Result<()> {
        let temp = self.clone();
        let delete = fs::remove_file("./hotkeys.json");
        match delete {
            Ok(_) => {}
            Err(_) => {}
        }
        let w_file = fs::File::options().read(true).write(true).create(true).open("./hotkeys.json")?;
        serde_json::to_writer(w_file, &temp)?;
        Ok(())
    }
    fn read_from_disk(&mut self) -> std::io::Result<()> {
        let file = fs::File::options().read(true).open("./hotkeys.json")?;
        let res: SaveHotKeys = serde_json::from_reader(file)?;
        *self = res;
        Ok(())
    }
    pub fn get_hotkey(&self) -> Code {
        self.key
    }
    pub fn set_hotkey(&mut self, key: Code) {
        self.key = key;
    }
    pub fn get_modifiers(&self) -> global_hotkey::hotkey::Modifiers {
        self.modifier
    }
    pub fn as_hotkey(&self) -> HotKey {
        HotKey::new(Some(self.get_modifiers()), self.get_hotkey())
    }
}


fn check_valid_shortcut(
    shortcuts: &HashMap<KeyCommand, VirtualShortcut>,
    test_key: Key,
    test_command: KeyCommand,
) -> Option<KeyCommand> {
    if test_command == KeyCommand::TakeScreenshot {
        return None;
    }
    for (command, shortcut) in shortcuts.iter() {
        if test_command != command.clone() && shortcut.key == test_key {
            return Some(command.clone());
        }
    }
    return None;
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ShortcutManager {
    shortcuts: HashMap<KeyCommand, VirtualShortcut>,
    global_shortcut: SaveHotKeys,
    changed_global_shortcut: bool,
    changed_settings: bool,
    last_extension: String,
    show_window: bool,
    waiting_for_input: bool,
    editing_command: KeyCommand,
    input_changed: bool,
    key_temp: Option<Key>,
    shortcut_invalid: Option<KeyCommand>,
    pub default_path: Option<PathBuf>,
    pub extension: String,
}

impl Default for ShortcutManager {
    fn default() -> Self {
        let mut map = HashMap::new();
        map.insert(
            KeyCommand::SaveScreenshot,
            VirtualShortcut::new(Modifiers::CTRL, Key::S),
        );
        map.insert(
            KeyCommand::Edit,
            VirtualShortcut::new(Modifiers::CTRL, Key::E),
        );
        map.insert(
            KeyCommand::Copy,
            VirtualShortcut::new(Modifiers::CTRL, Key::C),
        );
        map.insert(
            KeyCommand::QuickSaveScreenshot,
            VirtualShortcut::new(Modifiers::CTRL, Key::Q),
        );
        return Self {
            shortcuts: map,
            global_shortcut: SaveHotKeys::new(),
            changed_global_shortcut: false,
            changed_settings: false,
            last_extension: ".png".to_string(),
            show_window: false,
            waiting_for_input: false,
            editing_command: KeyCommand::None,
            input_changed: false,
            key_temp: None,
            shortcut_invalid: None,
            default_path: Some(PathBuf::from("./")),
            extension: ".png".to_string(),
        };
    }
}

fn write_to_disk(temp: &ShortcutManager) -> anyhow::Result<()> {
    temp.global_shortcut.write_to_disk()?;
    let delete = fs::remove_file("./settings.txt");
    match delete {
        Ok(_) => {}
        Err(_) => {}
    }
    let w_file = fs::File::options().read(true).write(true).create(true).open("./settings.txt")?;
    serde_json::to_writer(w_file, temp)?;
    Ok(())
}


fn read_from_disk() -> anyhow::Result<ShortcutManager> {
    let file = fs::File::options().read(true).open("./settings.txt")?;
    let res: ShortcutManager = serde_json::from_reader(file)?;
    Ok(res)
}

impl ShortcutManager {
    pub fn new() -> Self {
        let file_path = "./settings.txt";
        let res: Self = match fs::metadata(file_path) {
            Ok(_) => {
                match read_from_disk() {
                    Ok(mut res) => {
                        res.changed_global_shortcut = false;
                        res
                    }
                    Err(_) => {
                        ShortcutManager::default()
                    }
                }
            }
            Err(_) => {
                ShortcutManager::default()
            }
        };
        return res;
    }

    pub fn render_window(&mut self, ui: &mut Ui) {
        Window::new("Settings".to_string())
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
                            ui.add(egui::Separator::default());
                            egui::Grid::new("Invalid Shortcut").show(ui, |ui| {
                                ui.label("This shortcut is already in use for :");
                                ui.label(command.to_string());
                            });
                            ui.add(egui::Separator::default());
                        }
                        None => {}
                    }
                    ui.add(egui::Separator::default());
                    egui::Grid::new(self.waiting_for_input).show(ui, |ui| {
                        if ui.add(Button::new("Confirm")).clicked() {
                            match check_valid_shortcut(
                                &self.shortcuts,
                                self.key_temp.unwrap().clone(),
                                self.editing_command.clone(),
                            ) {
                                None => {
                                    if self.input_changed && self.editing_command != KeyCommand::TakeScreenshot {
                                        match self.shortcuts.get_mut(&self.editing_command) {
                                            Some(s) => {
                                                s.key = self.key_temp.unwrap();
                                            }
                                            None => {}
                                        }
                                    } else if self.editing_command == KeyCommand::TakeScreenshot {
                                        let old = self.global_shortcut.clone();
                                        self.global_shortcut.set_hotkey(VirtualKey::from_key(self.key_temp.unwrap()).to_hotkey());
                                        if old != self.global_shortcut {
                                            self.changed_global_shortcut = true;
                                        }
                                        match self.global_shortcut.write_to_disk() {
                                            Ok(_) => {}
                                            Err(_) => {}
                                        }
                                    }
                                    self.changed_settings = true;
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
                            columns[0].label(format!("{}", command));
                            columns[1].label(format!(
                                "CTRL + {}",
                                VirtualKey::from_key(shortcut.key.clone())
                            ));
                            columns[2].vertical_centered(|ui| {
                                if ui.add(Button::new("Edit")).clicked() {
                                    self.waiting_for_input = true;
                                    self.editing_command = command.clone();
                                    self.key_temp = Some(shortcut.key.clone());
                                }
                            });
                        });
                        ui.add(egui::Separator::default());
                    }

                    ui.columns(3, |columns| {
                        columns[0].label(format!("Screenshot"));
                        columns[1].label(format!(
                            "ALT + {}",
                            VirtualKey::from_hotkey(self.global_shortcut.get_hotkey())
                        ));
                        columns[2].vertical_centered(|ui| {
                            if ui.add(Button::new("Edit")).clicked() {
                                self.waiting_for_input = true;
                                self.editing_command = KeyCommand::TakeScreenshot;
                                self.key_temp = Some(VirtualKey::from_hotkey(self.global_shortcut.get_hotkey()).into());
                            }
                        });
                    });
                    if self.changed_global_shortcut {
                        ui.label("You have changed the global shortcut, please restart the application for the changes to take effect");
                    }
                    ui.add(egui::Separator::default());

                    ui.columns(2, |columns| {
                        columns[0].label(format!("{}", self.default_path.as_ref().unwrap().clone().as_path().display().to_string()));
                        columns[1].vertical_centered(|ui| {
                            if ui.add(Button::new("Change default Path")).clicked() {
                                self.changed_settings = true;
                                match FileDialog::new().pick_folder()
                                {
                                    Some(path) => {
                                        self.default_path = Some(path)
                                    }
                                    None => {}
                                }
                            }
                        });
                    });
                    ui.add(egui::Separator::default());
                    ui.columns(2, |columns| {
                        columns[0].label(format!("Default Extension"));
                        columns[1].vertical_centered(|ui| {
                            if ComboBox::from_id_source(2)
                                .width(50.0)
                                .selected_text(format!("{}", self.extension.as_str()[1..self.extension.len()].to_string().to_ascii_uppercase()))
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(&mut self.extension, ".png".to_string(), "PNG");
                                    ui.selectable_value(&mut self.extension, ".jpg".to_string(), "JPG");
                                    ui.selectable_value(&mut self.extension, ".gif".to_string(), "GIF");
                                    ui.selectable_value(&mut self.extension, ".bmp".to_string(), "BMP");
                                }).response.clicked() {
                                self.changed_settings = true;
                            };
                        });
                    });

                    ui.add(egui::Separator::default());
                    if self.changed_settings {
                        if ui.add(Button::new("Apply changes")).clicked() {
                            self.changed_settings = false;
                            let new_scm = ShortcutManager {
                                shortcuts: self.shortcuts.clone(),
                                global_shortcut: self.global_shortcut.clone(),
                                changed_global_shortcut: self.changed_global_shortcut,
                                changed_settings: false,
                                last_extension: self.extension.clone(),
                                show_window: false,
                                waiting_for_input: self.waiting_for_input,
                                editing_command: self.editing_command.clone(),
                                input_changed: self.input_changed,
                                key_temp: self.key_temp.clone(),
                                shortcut_invalid: self.shortcut_invalid.clone(),
                                default_path: self.default_path.clone(),
                                extension: self.extension.clone(),
                            };

                            match write_to_disk(&new_scm)
                            {
                                Ok(_) => {}
                                Err(_) => {}
                            };
                        }
                    }
                    else {
                        ui.add_enabled(false,Button::new("Apply changes"));
                    }
                }
            });
    }

    pub fn show_window(&mut self) {
        return self.show_window = true;
    }

    /// Use the shortcut linked to the KeyCommand passed to the function
    /// Return true if the shortcut is detected and false otherwise (or if the shortcut does not exist)
    pub fn use_shortcut(&mut self, ctx: &Context, command: &KeyCommand) -> bool {
        match self.shortcuts.get(command) {
            Some(shortcut) => ctx.input_mut(|i| i.consume_shortcut(&shortcut.clone().into())),
            None => false,
        }
    }
}

struct VirtualKey {
    key: Key,
}

impl Into<Key> for VirtualKey {
    fn into(self) -> Key {
        self.key
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct VirtualShortcut {
    key: Key,
    modifier: Modifiers,
}

impl Into<KeyboardShortcut> for VirtualShortcut {
    fn into(self) -> KeyboardShortcut {
        KeyboardShortcut {
            modifiers: self.modifier,
            key: self.key,
        }
    }
}

impl Into<KeyboardShortcut> for &VirtualShortcut {
    fn into(self) -> KeyboardShortcut {
        KeyboardShortcut {
            modifiers: self.modifier,
            key: self.key,
        }
    }
}

impl From<KeyboardShortcut> for VirtualShortcut {
    fn from(value: KeyboardShortcut) -> Self {
        Self {
            key: value.key,
            modifier: value.modifiers,
        }
    }
}

impl VirtualShortcut {
    fn new(modifier: Modifiers, key: Key) -> Self {
        Self {
            key,
            modifier,
        }
    }
}

impl Display for VirtualKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl VirtualKey {
    fn from_key(key: Key) -> Self {
        return Self { key };
    }
    fn to_string(&self) -> String {
        return format!("{}", self.key.name());
    }
    fn to_hotkey(self) -> Code {
        match self.key {
            Key::A => Code::KeyA,
            Key::B => Code::KeyB,
            Key::C => Code::KeyC,
            Key::D => Code::KeyD,
            Key::E => Code::KeyE,
            Key::F => Code::KeyF,
            Key::G => Code::KeyG,
            Key::H => Code::KeyH,
            Key::I => Code::KeyI,
            Key::J => Code::KeyJ,
            Key::K => Code::KeyK,
            Key::L => Code::KeyL,
            Key::M => Code::KeyM,
            Key::N => Code::KeyN,
            Key::O => Code::KeyO,
            Key::P => Code::KeyP,
            Key::Q => Code::KeyQ,
            Key::R => Code::KeyR,
            Key::S => Code::KeyS,
            Key::T => Code::KeyT,
            Key::U => Code::KeyU,
            Key::V => Code::KeyV,
            Key::W => Code::KeyW,
            Key::X => Code::KeyX,
            Key::Y => Code::KeyY,
            Key::Z => Code::KeyZ,
            Key::Num0 => Code::Digit0,
            Key::Num1 => Code::Digit1,
            Key::Num2 => Code::Digit2,
            Key::Num3 => Code::Digit3,
            Key::Num4 => Code::Digit4,
            Key::Num5 => Code::Digit5,
            Key::Num6 => Code::Digit6,
            Key::Num7 => Code::Digit7,
            Key::Num8 => Code::Digit8,
            Key::Num9 => Code::Digit9,
            Key::ArrowDown => Code::ArrowDown,
            Key::ArrowLeft => Code::ArrowLeft,
            Key::ArrowRight => Code::ArrowRight,
            Key::ArrowUp => Code::ArrowUp,
            _ => { Code::KeyT }
        }
    }
    fn from_hotkey(hotkey: Code) -> Self {
        let mut res = Self { key: Key::T };
        match hotkey {
            Code::Digit0 => res.key = Key::Num0,
            Code::Digit1 => res.key = Key::Num1,
            Code::Digit2 => res.key = Key::Num2,
            Code::Digit3 => res.key = Key::Num3,
            Code::Digit4 => res.key = Key::Num4,
            Code::Digit5 => res.key = Key::Num5,
            Code::Digit6 => res.key = Key::Num6,
            Code::Digit7 => res.key = Key::Num7,
            Code::Digit8 => res.key = Key::Num8,
            Code::Digit9 => res.key = Key::Num9,
            Code::KeyA => res.key = Key::A,
            Code::KeyB => res.key = Key::B,
            Code::KeyC => res.key = Key::C,
            Code::KeyD => res.key = Key::D,
            Code::KeyE => res.key = Key::E,
            Code::KeyF => res.key = Key::F,
            Code::KeyG => res.key = Key::G,
            Code::KeyH => res.key = Key::H,
            Code::KeyI => res.key = Key::I,
            Code::KeyJ => res.key = Key::J,
            Code::KeyK => res.key = Key::K,
            Code::KeyL => res.key = Key::L,
            Code::KeyM => res.key = Key::M,
            Code::KeyN => res.key = Key::N,
            Code::KeyO => res.key = Key::O,
            Code::KeyP => res.key = Key::P,
            Code::KeyQ => res.key = Key::Q,
            Code::KeyR => res.key = Key::R,
            Code::KeyS => res.key = Key::S,
            Code::KeyT => res.key = Key::T,
            Code::KeyU => res.key = Key::U,
            Code::KeyV => res.key = Key::V,
            Code::KeyW => res.key = Key::W,
            Code::KeyX => res.key = Key::X,
            Code::KeyY => res.key = Key::Y,
            Code::KeyZ => res.key = Key::Z,
            Code::ArrowDown => res.key = Key::ArrowDown,
            Code::ArrowLeft => res.key = Key::ArrowLeft,
            Code::ArrowRight => res.key = Key::ArrowRight,
            Code::ArrowUp => res.key = Key::ArrowUp,
            Code::Numpad0 => res.key = Key::Num0,
            Code::Numpad1 => res.key = Key::Num1,
            Code::Numpad2 => res.key = Key::Num2,
            Code::Numpad3 => res.key = Key::Num3,
            Code::Numpad4 => res.key = Key::Num4,
            Code::Numpad5 => res.key = Key::Num5,
            Code::Numpad6 => res.key = Key::Num6,
            Code::Numpad7 => res.key = Key::Num7,
            Code::Numpad8 => res.key = Key::Num8,
            Code::Numpad9 => res.key = Key::Num9,
            _ => { res.key = Key::T }
        }
        return res;
    }
}
