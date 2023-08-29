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
use image:: DynamicImage ;
use rfd::FileDialog;
use screenshots::DisplayInfo;
use std::borrow::Cow;
use std::cmp::max;
use std::collections::HashMap;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use std::time::Duration;

use self::shortcuts::ShortcutManager;

fn select_display(index: usize) -> Option<DisplayInfo> {
    let mydisp = screenshots::DisplayInfo::all();
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
    shape_window_open: bool,
}


impl RustShot {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.
        let (tx, rx) = channel();
        let (icons_map, tooltips_map) = load_icons();
        RustShot {
            curr_screenshot: None,
            display: Some(0),
            receiver: rx,
            sender: tx,
            paint_info: PaintState {
                curr_tool: Tool::None,
                curr_color: [255, 255, 255, 255],
                curr_thickness: 1,
                painting: false,
                last_ptr: Pos2::default(),
                curr_ptr: Pos2::default(),
                drawn_objects: Vec::new(),
            },
            action: Action::None,
            timer: Some(0),
            allowed_to_close: true,
            show_confirmation_dialog: false,
            shortcuts: ShortcutManager::default(),
            icons: icons_map,
            tooltips: tooltips_map,
            shape_window_open : false,
        }
    }

    fn render_top_panel(&mut self, ctx: &Context, frame: &mut Frame) {
        
        TopBottomPanel::top("top panel").show(ctx, |ui| {
            ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                self.shortcuts.render_window(ui);
                if self.action == Action::None {
                    let screenshot_btn = ui.add(Button::new("➕ New"));
                    let screenshot_save_btn = ui.add(Button::new("💾 Save"));
                    //Spawn edit only if screenshot is available
                    if self.curr_screenshot.is_some() {
                        let paint_btn = ui.add(Button::new("Edit"));
                        if paint_btn.clicked() || self.shortcuts.use_shortcut(ctx, &KeyCommand::Edit)
                        {
                            self.action = Action::Paint;
                        }
                    }
                    ComboBox::from_label("")
                        .width(80.0)
                        .selected_text(format!("🕓 {:?} sec", self.timer.unwrap()))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.timer, Some(0), "🕓 0 sec");
                            ui.selectable_value(&mut self.timer, Some(2), "🕓 2 sec");
                            ui.selectable_value(&mut self.timer, Some(5), "🕓 5 sec");
                            ui.selectable_value(&mut self.timer, Some(10), "🕓 10 sec");
                        });
                    self.display_selector(ui);
                    if screenshot_btn.clicked() || self.shortcuts.use_shortcut(ctx, &KeyCommand::TakeScreenshot)
                    {
                        self.store_screenshot(frame, ctx);
                    }

                    if screenshot_save_btn.clicked() || self.shortcuts.use_shortcut(ctx, &KeyCommand::SaveScreenshot)
                    {
                        match &self.curr_screenshot {
                            Some(screenshot) => {
                                save_screenshot(&screenshot.get_final_image().get_image());
                            }
                            None => {}
                        }
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
        CentralPanel::default().show(ctx, |ui| match &self.curr_screenshot {
            //If screenshot is already available, then show it on the GUI
            Some(screenshot) => {
                let screenshot = match self.action {
                    Action::None => screenshot.get_final_image().get_image(),
                    Action::Paint => screenshot.get_tmp_image().get_image(),
                };
                ScrollArea::both().show(ui, |ui| {
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
                        self.paint_logic(ctx, img);
                    }
                });
            }
            None => {
                ScrollArea::both().show(ui, |ui| ui.label("No screenshots yet"));
            }
        });
    }

    fn render_shape_window(&mut self, ctx:&Context, _ui:&mut Ui) {
        Window::new("Choose the shape").title_bar(false).
            show(ctx, |ui| {
                ui.group( |ui| {
                    ui.horizontal(|ui|  {
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
                    }});
                });
            });
    }

    /// Used to restore state of the screenshot when undoing paint changes
    fn restore_from_paint(&mut self) {
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

    fn copy_image(&mut self) {
        let mut clipboard = Clipboard::new().unwrap();
        let final_image = self.curr_screenshot.as_ref().unwrap().get_final_image().get_image();
        let bytes = final_image.as_bytes();
        let img = arboard::ImageData {
            width: final_image.width() as usize,
            height: final_image.height() as usize,
            bytes: Cow::from(bytes),
        };
        let done = clipboard.set_image(img);
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
        let mut selected = 0;
        ComboBox::from_id_source(0)
            .selected_text(format!("🖵 Display {:?}", self.display.unwrap()))
            .show_ui(ui, |ui| {
                for (i, display) in screen::display_list().into_iter().enumerate() {
                    if ui
                        .selectable_value(
                            &mut self.display.clone().unwrap(),
                            i,
                            format!(
                                "🖵 Display {}  {}x{}",
                                i,
                                display.width,
                                display.height,
                            ),
                        )
                        .clicked()
                    {
                        selected = i;
                        println!("Selected : {}", selected);
                        self.display = Some(selected);
                    }
                }
            });
    }

    /// Renders an ImageButton using the svg corresponding to the given name, if the svg failed to load or the name does not correspond to any svg, it spawns a button with the name passed as parameter to icon_button
    fn icon_button(&self, name: &str, enabled:bool, ctx: &Context, ui: &mut Ui) -> Response {
        match self.icons.get(name) {
            Some(val) => match val {
                Ok(image) => ui
                    .add_enabled(enabled, ImageButton::new(image.texture_id(ctx), image.size_vec2()))
                    .on_hover_text(self.tooltips.get(name).unwrap_or(&"Error".to_string())),
                Err(_) => ui.add_enabled(enabled,Button::new(name)),
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
            let save_paint_btn = ui.add(Button::new("Save changes"));
            //let save_paint_btn = ui.add_sized([100.0, 100.0],Button::new("Save changes"));

            if self.curr_screenshot.as_ref().unwrap().get_images_len() > 1 {
                let undo_btn = self.icon_button("arrow-90deg-left", true, ctx, ui);
                if undo_btn.clicked() {
                    let curr_screenshot = self.curr_screenshot.as_mut().unwrap();
                    let img = curr_screenshot.pop_last_image();
                    curr_screenshot.push_redo_image(img);
                    let img = curr_screenshot.get_last_image();
                    curr_screenshot.set_tmp_image(img);
                }
            }
            else {
                let undo_btn = self.icon_button("arrow-90deg-left",  false, ctx, ui);
            }
            if self.curr_screenshot.as_ref().unwrap().get_redo_images_len() > 0 {
                let redo_btn = self.icon_button("arrow-90deg-right", true, ctx, ui);
                if redo_btn.clicked() {
                    let curr_screenshot = self.curr_screenshot.as_mut().unwrap();
                    let img = curr_screenshot.pop_redo_image().unwrap();
                    curr_screenshot.stack_image(img.clone());
                    curr_screenshot.set_tmp_image(img);
                }
            }
            else {
                let redo_btn = self.icon_button("arrow-90deg-right", false, ctx, ui);
            }
            let draw_btn = self.icon_button("pencil-fill", true, ctx, ui);
            let highlighter_btn = self.icon_button("highlighter-solid", true, ctx, ui);
            let shape_btn = self.icon_button("pentagon", true, ctx, ui);
            let crop_btn = self.icon_button("crop", true, ctx, ui);
            let eraser_btn = self.icon_button("eraser-fill", true, ctx, ui);
            let rmv_tool_btn = self.icon_button("x-octagon", true, ctx, ui);
            ui.label("Current tool:");
            let curr_tool = match self.paint_info.curr_tool {
                Tool::Drawing => self.icon("pencil-fill", ctx, ui),
                Tool::HollowRect => self.icon("square", ctx, ui),
                Tool::FilledRect => self.icon("square-fill", ctx, ui),
                Tool::HollowCircle => self.icon("circle", ctx, ui),
                Tool::FilledCircle => self.icon("circle-fill", ctx, ui),
                Tool::Arrow => self.icon("arrow-up-right", ctx, ui),
                Tool::Eraser => self.icon("eraser-fill", ctx, ui),
                Tool::Highlighter => self.icon("highlighter-solid", ctx, ui),
                Tool::Crop => self.icon("crop", ctx, ui),
                Tool::None => ui.add(Label::new("None")),
            };
            if self.paint_info.curr_tool != Tool::None && self.paint_info.curr_tool != Tool::Crop && self.paint_info.curr_tool != Tool::Eraser{
                ui.color_edit_button_srgba_unmultiplied(&mut self.paint_info.curr_color);
            }
            if self.paint_info.curr_tool != Tool::None && self.paint_info.curr_tool != Tool::Crop{
                ui.add(Slider::new(&mut self.paint_info.curr_thickness, 0..=30));
            }
            if rmv_tool_btn.clicked() {
                self.paint_info.curr_tool = Tool::None;
            }
            if save_paint_btn.clicked() || self.shortcuts.use_shortcut(ctx, &KeyCommand::Edit) {
                self.action = Action::None;
                self.save_paint_changes();
            }
            if draw_btn.clicked() {
                self.paint_info.curr_tool = Tool::Drawing;
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
    fn paint_logic(&mut self, ctx: &Context, img: Response) {
        let curr_screenshot = self.curr_screenshot.as_mut().unwrap();
        if img.dragged() {
            if !self.paint_info.painting {
                self.paint_info.painting = true;
                self.paint_info.last_ptr =
                    into_relative_pos(img.interact_pointer_pos().unwrap(), img.rect);
            }
            self.paint_info.curr_ptr = match img.hover_pos() {
                Some(pos) => into_relative_pos(pos, img.rect),
                None => self.paint_info.last_ptr,
            };
            let mut screen_to_paint = curr_screenshot.get_last_image();
            match self.paint_info.curr_tool {
                Tool::Drawing => screen_to_paint = curr_screenshot.get_tmp_image(),
                Tool::Highlighter => screen_to_paint = curr_screenshot.get_tmp_image(),
                Tool::Eraser => screen_to_paint = curr_screenshot.get_tmp_image(),
                _ => {}
            }
            // When using Eraser, i need the latest clean version of the cropped image, when highlighting only the latest version of the image
            let tmp = match self.paint_info.curr_tool {
                Tool::Eraser => curr_screenshot.get_crop_image(screen_to_paint.get_crop_index()),
                Tool::Highlighter => curr_screenshot.get_last_image().get_image(),
                _ => curr_screenshot.get_last_image().get_image(),
            };
            self.paint_info.apply_tool(&mut screen_to_paint, tmp);
            if self.paint_info.curr_tool == Tool::Drawing || self.paint_info.curr_tool == Tool::Highlighter || self.paint_info.curr_tool == Tool::Eraser {
                self.paint_info.last_ptr = self.paint_info.curr_ptr;
            }
            curr_screenshot.set_tmp_image(screen_to_paint);
        } else if img.drag_released() {
            if self.paint_info.curr_tool == Tool::Crop {
                self.paint_info.curr_ptr =
                    into_relative_pos(img.interact_pointer_pos().unwrap(), img.rect);
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
                let img = editing_mod::Image::new(new_screen, curr_screenshot.get_crop_images_len());
                curr_screenshot.stack_image(img.clone());
                curr_screenshot.set_tmp_image(img);
                curr_screenshot.push_crop_image(crop_image);
            } else {
                curr_screenshot.stack_image(curr_screenshot.get_tmp_image());
            }
            self.paint_info.soft_reset();
        }
        //Change cursor when using a tool
        match self.paint_info.curr_tool {
            Tool::None => {}
            _ => {
                img.on_hover_cursor(CursorIcon::Crosshair);
            }
        }
    }
}

fn save_screenshot(screenshot: &DynamicImage) {
    let path =
        //tinyfiledialogs::save_file_dialog("Select save location", "./screen.jpg");
        FileDialog::new().add_filter("PNG", &["png"])
            .add_filter("JPG", &["jpg"]).add_filter("GIF", &["gif"])
            .add_filter("BMP", &["Bmp"])
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
    let window_option = NativeOptions::default();
    run_native(
        "RustShot",
        window_option,
        Box::new(|cc| Box::new(RustShot::new(cc))),
    )
}
