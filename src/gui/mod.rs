mod shortcuts;
mod image_proc_extra_mod;
mod editing_mod;
mod config_mod;

use crate::screen::{self, take_screenshot};
use crate::gui::image_proc_extra_mod::*;
use crate::gui::editing_mod::*;
use crate::gui::config_mod::*;

use eframe::egui::{Align, Button, CentralPanel, ColorImage, ComboBox, Context, CursorIcon, ImageButton, Label, Layout, Pos2, Response, ScrollArea, Sense, Slider, TopBottomPanel, Ui, Window};
use arboard::Clipboard;
use eframe::{run_native, NativeOptions};
use eframe::{App, Frame};
use egui_extras::RetainedImage;
use image::DynamicImage;
use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager};
use rfd::FileDialog;
use screenshots::DisplayInfo;
use std::borrow::Cow;
use std::cmp::max;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use std::time::{Duration, UNIX_EPOCH};
use eframe::emath::Rect;
use egui::{Event, Vec2};
use rusttype::{Font, Scale};

use self::shortcuts::{ShortcutManager, SaveHotKeys};

fn select_display(index: usize) -> Option<DisplayInfo> {
    let mydisp = DisplayInfo::all();
    match mydisp {
        Ok(disp) =>
            Some(disp[index]),

        Err(_) => None,
    }
}

#[derive(PartialEq, Eq)]
enum Action {
    Paint,
    None,
}

struct RustShot {
    curr_screenshot: Option<ImageStack>,
    display: Option<usize>,
    receiver: Receiver<DynamicImage>,
    sender: Sender<DynamicImage>,
    paint_info: PaintState,
    action: Action,
    timer: Option<u64>,
    show_confirmation_dialog: bool,
    allowed_to_close: bool,
    shortcuts: ShortcutManager,
    icons: HashMap<String, Result<RetainedImage, String>>,
    tooltips: HashMap<String, String>,
    fonts: HashMap<String, Option<Font<'static>>>,
    shape_window_open: bool,
    rx_global: Receiver<GlobalHotKeyEvent>,
}


impl RustShot {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.
        let (tx, rx) = channel();
        let (icons_map, tooltips_map) = load_icons();
        let fonts_map = load_fonts();
        let (tx_global,rx_global) = channel();
        let c = cc.egui_ctx.clone();
        
        thread::spawn(move || {
            loop {
                if let Ok(event) = GlobalHotKeyEvent::receiver().try_recv() {
                    match tx_global.send(event) {
                        Ok(_) => c.request_repaint(),
                        Err(err) => println!("{}", err),
                    }
                }
            }
        });
        
        RustShot {
            curr_screenshot: None,
            display: Some(0),
            receiver: rx,
            sender: tx,
            paint_info: PaintState::new(),
            action: Action::None,
            timer: Some(0),
            allowed_to_close: true,
            show_confirmation_dialog: false,
            shortcuts: ShortcutManager::new(),
            icons: icons_map,
            tooltips: tooltips_map,
            fonts: fonts_map,
            shape_window_open: false,
            rx_global
        }
    }

    fn render_top_panel(&mut self, ctx: &Context, frame: &mut Frame) {
        TopBottomPanel::top("top panel").show(ctx, |ui| {
            ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                self.shortcuts.render_window(ui);
                if self.action == Action::None {
                    let screenshot_btn = ui.add(Button::new("➕ New")).on_hover_text("Take screenshot of selected display");
                    //Spawn edit and save only if screenshot is available
                    if self.curr_screenshot.is_some() {
                        let screenshot_save_btn = ui.add(Button::new("💾 Save as")).on_hover_text("Save screenshot as");
                        if screenshot_save_btn.clicked() || self.shortcuts.use_shortcut(ctx, &KeyCommand::SaveScreenshot)
                        {
                            match &self.curr_screenshot {
                                Some(screenshot) => {
                                    self.save_screenshot(&screenshot.get_final_image().get_image());
                                }
                                None => {}
                            }
                        }

                        let screenshot_save_default_btn = ui.add(Button::new("💾 Save")).on_hover_text("Save screenshot in default path");
                        if screenshot_save_default_btn.clicked() || self.shortcuts.use_shortcut(ctx, &KeyCommand::QuickSaveScreenshot)
                        {
                            match &self.curr_screenshot {
                                Some(screenshot) => {
                                    self.save_default_screenshot(&screenshot.get_final_image().get_image());
                                }
                                None => {}
                            }
                        }
                        let paint_btn = ui.add(Button::new("Edit")).on_hover_text("Edit screenshot");
                        if paint_btn.clicked() || self.shortcuts.use_shortcut(ctx, &KeyCommand::Edit)
                        {
                            self.action = Action::Paint;
                        }
                    }
                    ComboBox::from_label("")
                        .width(80.0)
                        .selected_text(format!("🕓 {:?} sec", self.timer.unwrap()))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.timer, Some(0), "🕓 0 sec").on_hover_text("Delay screenshot");
                            ui.selectable_value(&mut self.timer, Some(2), "🕓 2 sec").on_hover_text("Delay screenshot");
                            ui.selectable_value(&mut self.timer, Some(5), "🕓 5 sec").on_hover_text("Delay screenshot");
                            ui.selectable_value(&mut self.timer, Some(10), "🕓 10 sec").on_hover_text("Delay screenshot");
                        });
                    self.display_selector(ui);
                    if screenshot_btn.clicked() || self.shortcuts.use_shortcut(ctx, &KeyCommand::TakeScreenshot)
                    {
                        self.store_screenshot(frame, ctx);
                    }

                    let setting_btn = self.icon_button("gear", true, ctx, ui);
                    if setting_btn.clicked() {
                        self.shortcuts.show_window();
                    }
                    //Spawn clipboard only if screenshot is already available
                    if self.curr_screenshot.is_some() {
                        let copy_btn = self.icon_button("clipboard", true, ctx, ui);
                        if copy_btn.clicked() || self.shortcuts.use_shortcut(ctx, &KeyCommand::Copy) {
                            self.copy_image();
                        }
                    }
                } else if self.action == Action::Paint {
                    self.render_paint_tools(ctx, ui);
                }
            })
        });
    }

    fn render_central_panel(&mut self, ctx: &Context, _frame: &mut Frame) {
        CentralPanel::default().show(ctx, |ui| match &mut self.curr_screenshot {
            //If screenshot is already available, then show it on the GUI
            Some(screenshot) => {
                let screenshot = match self.action {
                    Action::None => screenshot.get_final_image().get_image(),
                    Action::Paint => screenshot.get_tmp_image().get_image(),
                };
                ScrollArea::both().show_viewport(ui, |ui, rect| {
                    let retained_img = RetainedImage::from_color_image(
                        "screenshot",
                        ColorImage::from_rgba_unmultiplied(
                            [screenshot.width() as usize, screenshot.height() as usize],
                            screenshot.as_bytes(),
                        ),
                    );
                    let img = ui.add(
                        ImageButton::new(retained_img.texture_id(ctx), retained_img.size_vec2())
                            .frame(false)
                            .sense(Sense::click_and_drag()),
                    );
                    if self.action == Action::Paint {
                        self.paint_logic(img, ui, rect);
                    }
                });
            }
            None => {
                ScrollArea::both().show(ui, |ui| ui.label("No screenshots yet"));
            }
        });
    }

    fn render_shape_window(&mut self, ctx: &Context, _ui: &mut Ui) {
        Window::new("Choose the shape").title_bar(false).
            show(ctx, |ui| {
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        let hollow_rect_btn = self.icon_button("square", true, ctx, ui);
                        let filled_rect_btn = self.icon_button("square-fill", true, ctx, ui);
                        let hollow_circle_btn = self.icon_button("circle", true, ctx, ui);
                        let filled_circle_btn = self.icon_button("circle-fill", true, ctx, ui);
                        let arrow_btn = self.icon_button("arrow-up-right", true, ctx, ui);
                        if hollow_rect_btn.clicked() {
                            self.paint_info.curr_tool = Tool::HollowRect;
                        }
                        if filled_rect_btn.clicked() {
                            self.paint_info.curr_tool = Tool::FilledRect;
                        }
                        if hollow_circle_btn.clicked() {
                            self.paint_info.curr_tool = Tool::HollowCircle;
                        }
                        if filled_circle_btn.clicked() {
                            self.paint_info.curr_tool = Tool::FilledCircle;
                        }
                        if arrow_btn.clicked() {
                            self.paint_info.curr_tool = Tool::Arrow;
                        }
                        let close_btn = self.icon_button("x", true, ctx, ui);
                        if close_btn.clicked() {
                            self.shape_window_open = false;
                        }
                    });
                });
            });
    }

    /// Used to restore state of the screenshot when undoing paint changes
    fn _restore_from_paint(&mut self) {
        self.paint_info.reset();
        //Restore the original screenshot
        if self.curr_screenshot.is_some() {
            self.curr_screenshot.as_mut().unwrap().restore();
        }
    }

    fn save_paint_changes(&mut self) {
        self.paint_info.reset();
        //Save the changed screenshot as final screenshot
        if self.curr_screenshot.is_some() {
            self.curr_screenshot.as_mut().unwrap().save_changes();
        }
    }

    fn undo_paint_changes(&mut self) {
        self.paint_info.reset();
        if self.curr_screenshot.is_some() {
            self.curr_screenshot.as_mut().unwrap().undo_changes();
        }
    }

    fn copy_image(&mut self) {
        let mut clipboard = Clipboard::new().unwrap();
        let final_image = self.curr_screenshot.as_ref().unwrap().get_final_image().get_image();
        let bytes = final_image.as_bytes();
        let img = arboard::ImageData {
            width: final_image.width() as usize,
            height: final_image.height() as usize,
            bytes: Cow::from(bytes),
        };
        let _done = clipboard.set_image(img);
    }

    fn store_screenshot(&mut self, frame: &mut Frame, ctx: &Context) {
        //Hide the application window
        self.allowed_to_close = false;
        frame.set_visible(false);

        let tx = self.sender.clone();
        let c = ctx.clone();
        let timer = self.timer.unwrap().clone();
        let value = self.display.unwrap().clone();
        println!("Display : {}", value);
        //Thread that manages screenshots
        thread::spawn(move || {
            thread::sleep(Duration::from_millis(timer * 1000 + 300));
            let current_display: DisplayInfo = select_display(value as usize)
                .expect("Cannot select the correct display");
            let screenshot = take_screenshot(&current_display).unwrap();
            println!("screenshot done");
            match tx.send(screenshot) {
                //Force update() to be called again, so that the application window is made visible again. (when it's not visible otherwise update won't be called)
                Ok(_) => c.request_repaint(),
                Err(err) => println!("{}", err),
            }
        });
    }

    fn display_selector(&mut self, ui: &mut Ui) {
        ComboBox::from_id_source(0)
            .selected_text(format!("🖵 Display {:?}", self.display.unwrap()))
            .show_ui(ui, |ui| {
                for (i, display) in screen::display_list().iter().enumerate(){
                    ui.selectable_value(&mut self.display, Some(i), format!("🖵 Display {}  {}x{}", i, display.width, display.height))
                        .on_hover_text("Select display");
                }

            });
    }

    /// Renders an ImageButton using the svg corresponding to the given name, if the svg failed to load or the name does not correspond to any svg, it spawns a button with the name passed as parameter to icon_button
    fn icon_button(&self, name: &str, enabled: bool, ctx: &Context, ui: &mut Ui) -> Response {
        match self.icons.get(name) {
            Some(val) => match val {
                Ok(image) => ui
                    .add_enabled(enabled, ImageButton::new(image.texture_id(ctx), image.size_vec2()))
                    .on_hover_text(self.tooltips.get(name).unwrap_or(&"Error".to_string())),
                Err(_) => ui.add_enabled(enabled, Button::new(name)),
            },
            None => ui.add_enabled(enabled, Button::new(name)),
        }
    }

    /// Renders an icon using the svg corresponding to the given name, if the svg failed to load or the name does not correspond to any svg, it spawns a button with the name passed as parameter to icon
    fn icon(&self, name: &str, ctx: &Context, ui: &mut Ui) -> Response {
        match self.icons.get(name) {
            Some(val) => match val {
                Ok(image) => ui
                    .image(image.texture_id(ctx), image.size_vec2())
                    .on_hover_text(self.tooltips.get(name).unwrap_or(&"Error".to_string())),
                Err(_) => ui.add(Label::new(name)),
            },
            None => ui.add(Label::new(name)),
        }
    }

    /// Renders painting annotation tools when in paint mode
    fn render_paint_tools(&mut self, ctx: &Context, ui: &mut Ui) {
        if self.shape_window_open {
            self.render_shape_window(ctx, ui);
        }
        ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
            let undo_changes_btn = ui.add(Button::new("Exit"));
            let save_paint_btn = ui.add(Button::new("Save changes"));
            //let save_paint_btn = ui.add_sized([100.0, 100.0],Button::new("Save changes"));

            if self.curr_screenshot.as_ref().unwrap().get_images_len() > 1 && self.paint_info.curr_tool != Tool::Text {
                let undo_btn = self.icon_button("arrow-90deg-left", true, ctx, ui);
                if undo_btn.clicked() {
                    let curr_screenshot = self.curr_screenshot.as_mut().unwrap();
                    let img = curr_screenshot.pop_last_image();
                    curr_screenshot.push_redo_image(img);
                    let img = curr_screenshot.get_last_image();
                    curr_screenshot.set_tmp_image(img);
                }
            } else {
                let _undo_btn = self.icon_button("arrow-90deg-left", false, ctx, ui);
            }
            if self.curr_screenshot.as_ref().unwrap().get_redo_images_len() > 0 && self.paint_info.curr_tool != Tool::Text {
                let redo_btn = self.icon_button("arrow-90deg-right", true, ctx, ui);
                if redo_btn.clicked() {
                    let curr_screenshot = self.curr_screenshot.as_mut().unwrap();
                    let img = curr_screenshot.pop_redo_image().unwrap();
                    curr_screenshot.stack_image(img.clone());
                    curr_screenshot.set_tmp_image(img);
                }
            } else {
                let _redo_btn = self.icon_button("arrow-90deg-right", false, ctx, ui);
            }
            let draw_btn = self.icon_button("pencil-fill", true, ctx, ui);
            let text_btn = self.icon_button("fonts", true, ctx, ui);
            let highlighter_btn = self.icon_button("highlighter-solid", true, ctx, ui);
            let shape_btn = self.icon_button("pentagon", true, ctx, ui);
            let crop_btn = self.icon_button("crop", true, ctx, ui);
            let eraser_btn = self.icon_button("eraser-fill", true, ctx, ui);
            let rmv_tool_btn = self.icon_button("x-octagon", true, ctx, ui);
            ui.label("Current tool:");
            let _curr_tool = match self.paint_info.curr_tool {
                Tool::Drawing => self.icon("pencil-fill", ctx, ui),
                Tool::HollowRect => self.icon("square", ctx, ui),
                Tool::FilledRect => self.icon("square-fill", ctx, ui),
                Tool::HollowCircle => self.icon("circle", ctx, ui),
                Tool::FilledCircle => self.icon("circle-fill", ctx, ui),
                Tool::Arrow => self.icon("arrow-up-right", ctx, ui),
                Tool::Eraser => self.icon("eraser-fill", ctx, ui),
                Tool::Highlighter => self.icon("highlighter-solid", ctx, ui),
                Tool::Crop => self.icon("crop", ctx, ui),
                Tool::Text => self.icon("fonts", ctx, ui),
                Tool::None => ui.add(Label::new("None")),
            };
            if self.paint_info.curr_tool != Tool::None && self.paint_info.curr_tool != Tool::Crop && self.paint_info.curr_tool != Tool::Eraser {
                ui.color_edit_button_srgba_unmultiplied(&mut self.paint_info.curr_color);
            }
            if self.paint_info.curr_tool != Tool::None && self.paint_info.curr_tool != Tool::Crop && self.paint_info.curr_tool != Tool::Text {
                ui.add(Slider::new(&mut self.paint_info.curr_thickness, 0..=30));
            }
            else if self.paint_info.curr_tool == Tool::Text {
                ui.add(Slider::new(&mut self.paint_info.text_info.curr_dim, 0..=60));
                ComboBox::from_label("Font")
                    .selected_text(self.paint_info.text_info.curr_font_name.clone())
                    .show_ui(ui, |ui| {
                        ui.style_mut().wrap = Some(false);
                        ui.set_min_width(60.0);
                        for s in self.fonts.keys() {
                            ui.selectable_value(&mut self.paint_info.text_info.curr_font_name, s.clone(), s);
                        }
                    });
                //If the font was not correctly loaded, keep the old one to avoid panic
                self.paint_info.text_info.curr_font = match self.fonts.get(self.paint_info.text_info.curr_font_name.as_str()) {
                    Some(font) => font.clone(),
                    None => self.paint_info.text_info.curr_font.clone(),
                }
            }
            if rmv_tool_btn.clicked() {
                self.paint_info.curr_tool = Tool::None;
            }
            if save_paint_btn.clicked() || self.shortcuts.use_shortcut(ctx, &KeyCommand::Edit) {
                self.action = Action::None;
                self.save_paint_changes();
            }
            if undo_changes_btn.clicked() {
                self.action = Action::None;
                self.undo_paint_changes();
            }
            if draw_btn.clicked() {
                self.paint_info.curr_tool = Tool::Drawing;
            }
            if text_btn.clicked() {
                //Go to text mode only if the default font has been loaded correctly
                if self.paint_info.text_info.curr_font.is_some() {
                    self.paint_info.curr_tool = Tool::Text;
                }
            }
            if shape_btn.clicked() {
                self.shape_window_open = true;
            }
            if highlighter_btn.clicked() {
                self.paint_info.curr_tool = Tool::Highlighter;
            }
            if crop_btn.clicked() {
                self.paint_info.curr_tool = Tool::Crop;
            }
            if eraser_btn.clicked() {
                self.paint_info.curr_tool = Tool::Eraser;
            }
        });
    }

    /// Logic for painting on the image
    fn paint_logic(&mut self, img: Response, ui: &mut Ui, rect: Rect) {
        let curr_screenshot = self.curr_screenshot.as_mut().unwrap();
        //If the tool is not Text, and text_info is dirty (update tmp_image so that it gets the changes without the temporary cursor and textarrea)
        if self.paint_info.curr_tool != Tool::Text && self.paint_info.text_info.dirty {
            self.paint_info.text_info.reset();
            //Set tmp img in order to delete the text area
            curr_screenshot.set_tmp_image(curr_screenshot.get_last_image());
        }
        if self.paint_info.curr_tool == Tool::Text && !self.paint_info.text_info.writing {
            match img.interact_pointer_pos() {
                Some(ptr) => {
                    //Initialization for the pop/stack in the following else if
                    curr_screenshot.stack_image(curr_screenshot.get_last_image());
                    self.paint_info.text_info.original_img = curr_screenshot.get_last_image();
                    self.paint_info.text_info.dirty = true;
                    self.paint_info.text_info.writing = true;
                    self.paint_info.text_info.edge = into_relative_pos(ptr, img.rect);
                    self.paint_info.text_info.height = self.paint_info.text_info.curr_dim as f32;
                    //Compute max_width/max_height when initializing a textarea
                    self.paint_info.text_info.max_width = curr_screenshot.tmp_image.get_width() as f32 - self.paint_info.text_info.edge.x;
                    self.paint_info.text_info.max_height = curr_screenshot.tmp_image.get_height() as f32 - self.paint_info.text_info.edge.y;
                }
                None => {}
            }
        }
        else if self.paint_info.curr_tool == Tool::Text && self.paint_info.text_info.writing {
            // I need to keep updating to iterate over the events
            ui.ctx().request_repaint();

            let mut screen_to_paint = self.paint_info.text_info.original_img.clone();
            let mut screen_to_paint_real = self.paint_info.text_info.original_img.clone();

            self.paint_info.curr_tool = Tool::Text;
            self.paint_info.apply_tool(&mut screen_to_paint_real, Some(self.paint_info.text_info.original_img.get_image()));

            // This is the real one, without textarea
            curr_screenshot.pop_last_image();
            curr_screenshot.stack_image(screen_to_paint_real.clone());

            //Draw current cursor
            let cursor = self.paint_info.text_info.cursor_x;
            self.paint_info.text_info.curr_str.insert(cursor, char::from_str("_").unwrap());
            self.paint_info.apply_tool(&mut screen_to_paint, Some(self.paint_info.text_info.original_img.get_image()));

            //Retrieve actual width and height of current textarea.
            let lines: Vec<&str> = self.paint_info.text_info.curr_str.split("\n").collect();
            self.paint_info.text_info.height = (lines.len() * self.paint_info.text_info.curr_dim as usize) as f32;
            let mut width = 0.;
            for l in &lines {
                let t = measure_line(self.paint_info.text_info.curr_font.as_ref().unwrap(), l, Scale::uniform(self.paint_info.text_info.curr_dim as f32));
                if t.0 >= width{
                    width = t.0;
                }
            }
            self.paint_info.text_info.width = width;

            //Draw the textarea
            let old_color = self.paint_info.curr_color;
            self.paint_info.curr_tool = Tool::HollowRect;
            self.paint_info.curr_thickness = 0;
            self.paint_info.curr_color = [0u8, 0u8, 0u8, 0u8];
            self.paint_info.last_ptr = Pos2::new(self.paint_info.text_info.edge.x - (self.paint_info.text_info.curr_dim/4) as f32, self.paint_info.text_info.edge.y - (self.paint_info.text_info.curr_dim/4) as f32);
            self.paint_info.curr_ptr = Pos2::new(self.paint_info.last_ptr.x + self.paint_info.text_info.width + (self.paint_info.text_info.curr_dim/4 + self.paint_info.text_info.curr_dim/4) as f32, self.paint_info.last_ptr.y + self.paint_info.text_info.height + (self.paint_info.text_info.curr_dim/4 + self.paint_info.text_info.curr_dim/4) as f32);
            self.paint_info.apply_tool(&mut screen_to_paint, Some(self.paint_info.text_info.original_img.get_image()));
            self.paint_info.curr_tool = Tool::Text;
            self.paint_info.curr_color = old_color;

            //Remove the added cursor from the curr_str
            self.paint_info.text_info.curr_str.remove(cursor);

            curr_screenshot.set_tmp_image(screen_to_paint);
            //Logic for updating the state of text_info
            ui.input(|i| {
                let events = &i.events;
                for e in events {
                    match e {
                        Event::Text(str) => {
                            if self.paint_info.text_info.width < self.paint_info.text_info.max_width  {
                                self.paint_info.text_info.curr_str.insert_str(self.paint_info.text_info.cursor_x, str);
                                self.paint_info.text_info.cursor_x += str.len();
                            }
                        }
                        Event::Key { key, pressed, .. } => {
                            if let egui::Key::Enter = key  {
                                println!("height:{}, max_height:{}", self.paint_info.text_info.height, self.paint_info.text_info.max_height);
                                if *pressed && self.paint_info.text_info.height < self.paint_info.text_info.max_height {
                                    self.paint_info.text_info.curr_str.insert_str(self.paint_info.text_info.cursor_x, "\n");
                                    self.paint_info.text_info.cursor_x += 1;
                                }
                            }
                            else if let egui::Key::Backspace = key  {
                                if *pressed {
                                    if self.paint_info.text_info.cursor_x > 0{
                                        self.paint_info.text_info.curr_str.remove(self.paint_info.text_info.cursor_x - 1 );
                                        self.paint_info.text_info.cursor_x -= 1;
                                    }
                                }
                            }
                            else if let egui::Key::ArrowRight = key  {
                                if *pressed && self.paint_info.text_info.cursor_x < self.paint_info.text_info.curr_str.len(){
                                    self.paint_info.text_info.cursor_x += 1;
                                }
                            }
                            else if let egui::Key::ArrowLeft = key  {
                                if *pressed && self.paint_info.text_info.cursor_x > 0{
                                    self.paint_info.text_info.cursor_x -= 1;
                                }
                            }
                            else if let egui::Key::Escape = key {
                                self.paint_info.curr_tool = Tool::None;
                            }
                        }

                        _ => {}
                    }
                }
            }
            );
            //Stop writing text if click happens somewhere
             if img.drag_started(){
                 self.paint_info.curr_tool = Tool::None;
             }
        } else {
            if img.dragged() && self.paint_info.curr_tool != Tool::None {
                if !self.paint_info.painting {
                    match img.hover_pos(){
                        None => {}
                        Some(pos) => {
                            self.paint_info.last_ptr = into_relative_pos(pos, img.rect);
                            self.paint_info.painting = true;
                        }
                    }
                }
                self.paint_info.curr_ptr = match img.hover_pos() {
                    Some(pos) => into_relative_pos(pos, img.rect),
                    None => self.paint_info.curr_ptr,
                };

                // Automatic scrolling when using crop tool
                if self.paint_info.curr_tool == Tool::Crop {
                    if self.paint_info.curr_ptr.x >= rect.right() - 20. {
                        ui.scroll_with_delta(Vec2::new(rect.right() - 20. - self.paint_info.curr_ptr.x, 0.));
                    }
                    if self.paint_info.curr_ptr.x <= rect.left() + 20. {
                        ui.scroll_with_delta(Vec2::new(rect.left() + 20. - self.paint_info.curr_ptr.x, 0.));
                    }
                    if self.paint_info.curr_ptr.y <= rect.top() + 20. {
                        ui.scroll_with_delta(Vec2::new(0., rect.top() + 20. - self.paint_info.curr_ptr.y));
                    }
                    if self.paint_info.curr_ptr.y >= rect.bottom() - 20. {
                        ui.scroll_with_delta(Vec2::new(0., rect.bottom() - 20. - self.paint_info.curr_ptr.y));
                    }
                    // To make scrolling while cropping more fluid, i need to keep requesting to repaint
                    ui.ctx().request_repaint();
                }

                // When using Eraser, i need the latest clean version of the cropped image, when highlighting only the latest version of the image
                // Using Option, I avoid useless "clone()" when not needed with other tools, using get_image_as_ref, i avoid other useless "clone()".
                let tmp = match self.paint_info.curr_tool {
                    Tool::Eraser => Some(curr_screenshot.get_crop_image(curr_screenshot.get_last_image_as_ref().get_crop_index())),
                    Tool::Highlighter => Some(curr_screenshot.get_last_image_as_ref().get_decompressed_image()),
                    _ => None,
                };

                if self.paint_info.curr_tool == Tool::Drawing || self.paint_info.curr_tool == Tool::Highlighter || self.paint_info.curr_tool == Tool::Eraser {
                    self.paint_info.apply_tool(&mut curr_screenshot.tmp_image, tmp);
                    // This is needed for this tools, that act like continous lines
                    self.paint_info.last_ptr = self.paint_info.curr_ptr;
                }
                else {
                    let mut screen_to_paint = curr_screenshot.get_last_image();
                    self.paint_info.apply_tool(&mut screen_to_paint, tmp);
                    curr_screenshot.set_tmp_image(screen_to_paint);
                }
            } else if img.drag_released() && self.paint_info.curr_tool != Tool::None {
                if self.paint_info.curr_tool == Tool::Crop {
                    //self.paint_info.curr_ptr =
                    //    into_relative_pos(img.interact_pointer_pos().unwrap(), img.rect);
                    let width = max(
                        1,
                        (self.paint_info.curr_ptr.x - self.paint_info.last_ptr.x).abs() as i32,
                    );
                    let height = max(
                        1,
                        (self.paint_info.curr_ptr.y - self.paint_info.last_ptr.y).abs() as i32,
                    );
                    //Permits an easier selection when cropping, allowing to generate the crop area in all directions
                    let mut start_ptr = self.paint_info.last_ptr;
                    if self.paint_info.curr_ptr.x < self.paint_info.last_ptr.x {
                        start_ptr.x = self.paint_info.curr_ptr.x;
                    }
                    if self.paint_info.curr_ptr.y < self.paint_info.last_ptr.y {
                        start_ptr.y = self.paint_info.curr_ptr.y;
                    }
                    let curr_img = curr_screenshot.get_last_image();
                    let new_screen = curr_img.get_image().crop_imm(
                        start_ptr.x as u32,
                        start_ptr.y as u32,
                        width as u32,
                        height as u32,
                    );
                    let crop_image = curr_screenshot.get_crop_image(curr_img.get_crop_index()).crop_imm(
                        start_ptr.x as u32,
                        start_ptr.y as u32,
                        width as u32,
                        height as u32,
                    );
                    let img = Image::new(new_screen, curr_screenshot.get_crop_images_len());
                    curr_screenshot.stack_image(img.clone());
                    curr_screenshot.set_tmp_image(img);
                    curr_screenshot.push_crop_image(crop_image);
                } else {
                    curr_screenshot.stack_image(curr_screenshot.get_tmp_image());
                }
                self.paint_info.soft_reset();
            }
        }
        //Change cursor when using a tool
        match self.paint_info.curr_tool {
            Tool::None => {}
            Tool::Text => {
                if !self.paint_info.text_info.writing{
                    img.on_hover_cursor(CursorIcon::Text);
                }
            }
            _ => {
                img.on_hover_cursor(CursorIcon::Crosshair);
            }
        }
    }

    fn save_screenshot(&mut self,screenshot: &DynamicImage) {
        let path =
            //tinyfiledialogs::save_file_dialog("Select save location", "./screen.jpg");
            FileDialog::new().add_filter("PNG", &["png"])
                .add_filter("JPG", &["jpg"]).add_filter("GIF", &["gif"])
                .add_filter("BMP", &["bmp"])
                .set_directory("./")
                .save_file();
        match path {
            Some(path) => {
                match image::save_buffer(
                    path,
                    &screenshot.as_bytes(),
                    screenshot.width() as u32,
                    screenshot.height() as u32,
                    image::ColorType::Rgba8,
                ) {
                    Ok(_) => println!("Screenshot saved"),
                    Err(err) => println!("{}", err),
                }
            }
            None => {}
        }
    }


    fn save_default_screenshot(&mut self,screenshot: &DynamicImage) {
        let mut path = String::from(self.shortcuts.default_path.clone().unwrap().into_os_string().to_str().unwrap());
        path.push_str("/screen");
        let time = match std::time::SystemTime::now().duration_since(UNIX_EPOCH)
        {
            Ok(time_scr)=> time_scr.as_secs().to_string(),
            Err(_) => "".to_string(),
        };

        path = path + time.as_str() + self.shortcuts.extension.clone().as_str();
        match image::save_buffer(
            //self.default_path.unwrap()
            path,
            &screenshot.as_bytes(),
            screenshot.width() as u32,
            screenshot.height() as u32,
            image::ColorType::Rgba8,
        ) {
            Ok(_) => println!("Screenshot saved"),
            Err(err) => println!("{}", err),
        }
    }

}



impl App for RustShot {
    fn update(&mut self, ctx: &Context, frame: &mut Frame) {
        match self.receiver.try_recv() {
            Ok(screenshot) => {
                //Show the application window again
                frame.set_visible(true);
                //let color_image = ColorImage::from_rgb([screenshot.width() as usize, screenshot.height() as usize], screenshot.as_bytes());
                //self.screenshot = Some(RetainedImage::from_color_image("screenshot", color_image));
                self.curr_screenshot = Some(ImageStack::new(screenshot));
            }
            Err(_) => {}
        }
        match self.rx_global.try_recv() {
            Ok(_) => {
                self.store_screenshot(frame, ctx);
            }
            Err(_) => {}
        }
        if let Ok(event) = GlobalHotKeyEvent::receiver().try_recv() {
            println!("tray event: {event:?}");
        }
        self.render_top_panel(ctx, frame);
        self.render_central_panel(ctx, frame);
        if self.show_confirmation_dialog {
            // Show confirmation dialog:
            Window::new("Do you want to quit?")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        if ui.button("Cancel").clicked() {
                            self.show_confirmation_dialog = false;
                        }

                        if ui.button("Yes!").clicked() {
                            self.allowed_to_close = true;
                            frame.close();
                        }
                    });
                });
        }
    }
    fn on_close_event(&mut self) -> bool {
        if !self.allowed_to_close {
            self.show_confirmation_dialog = true;
        }
        self.allowed_to_close
    }
}


pub fn main_window() -> eframe::Result<()> {
    let manager = GlobalHotKeyManager::new().unwrap();

    let hotkey = SaveHotKeys::new();
    let hotkey = hotkey.as_hotkey();

    /* Register the global hotkey to take screenshot - works only here */
    manager.register(hotkey).unwrap();


    let window_option = NativeOptions::default();
    run_native(
        "RustShot",
        window_option,
        Box::new(|cc| {
            Box::new(RustShot::new(cc))
        }),
    )
}