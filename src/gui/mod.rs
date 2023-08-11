mod image_proc_extra_mod;
mod editing_mod;

use crate::screen::{self, take_screenshot};
use crate::gui::image_proc_extra_mod::*;
use crate::gui::editing_mod::*;

use eframe::egui::{Align, Button, CentralPanel, ColorImage, ComboBox, Context, CursorIcon, ImageButton, InputState, Key, KeyboardShortcut, Label, Layout, Modifiers, Pos2, Rect, Response, ScrollArea, Sense, Shape, Slider, TextureId, TopBottomPanel, Ui, Vec2, Window};
use arboard::Clipboard;
use eframe::{run_native, NativeOptions};
use eframe::{App, Frame};
use egui_extras::RetainedImage;
use image::{DynamicImage, Rgb, RgbImage, Rgba, GenericImage, Pixel, GenericImageView};
use imageproc::definitions::Image;
use imageproc::drawing;
use rfd::FileDialog;
use screenshots::DisplayInfo;
use std::borrow::Cow;
use std::cmp::max;
use std::collections::HashMap;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use std::time::Duration;
use imageproc::drawing::{BresenhamLineIter, Canvas};

fn select_display(index: usize) -> Option<DisplayInfo> {
    /*let mut iter = screen::display_list().into_iter().enumerate();
    return loop {
        match iter.next() {
            Some((i, display)) => {
                if i == index {
                    println!("i : {}", i);
                    break Some(display);
                }
                continue;
            }
            None => {
                break None;
            }
        }
    };*/

    let mydisp = screenshots::DisplayInfo::all();
    match mydisp {
        Ok(disp) =>
            Some(disp[index]),

        Err(_) => None,
    }
}

#[derive(PartialEq, Eq)]
enum Action {
    Crop,
    Paint,
    None,
}


#[derive(PartialEq, Eq, PartialOrd, Ord, Hash)]
enum KeyCommand {
    SaveScreenshot,
    TakeScreenshot,
    Crop,
    Paint,
    None,
}



struct RustShot {
    screenshot: Option<DynamicImage>,
    intermediate_screenshot: Option<DynamicImage>,
    final_screenshot: Option<DynamicImage>,
    display: Option<usize>,
    receiver: Receiver<DynamicImage>,
    sender: Sender<DynamicImage>,
    crop_info: CropState,
    paint_info: PaintState,
    action: Action,
    timer: Option<u64>,
    show_confirmation_dialog: bool,
    allowed_to_close: bool,
    shortcuts: HashMap<KeyCommand, KeyboardShortcut>,
    icons: HashMap<String, Result<RetainedImage, String>>,
    tooltips: HashMap<String, String>,
}

/// Load in the application state the svg icons as RetainedImage, and also the correspondence between the backend name of the icon and its tooltip.
fn load_icons() -> (
    HashMap<String, Result<RetainedImage, String>>,
    HashMap<String, String>,
) {
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
        "pen-fill".to_string(),
        RetainedImage::from_svg_bytes("pen-fill", include_bytes!("../../resources/pen-fill.svg")),
    );
    tooltips_map.insert("pen-fill".to_string(), "Highlight".to_string());
    return (icons_map, tooltips_map);
}

impl RustShot {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.
        let (tx, rx) = channel();
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
            KeyCommand::Paint,
            KeyboardShortcut {
                modifiers: Modifiers::CTRL,
                key: Key::P,
            },
        );
        let (icons_map, tooltips_map) = load_icons();
        RustShot {
            screenshot: None,
            final_screenshot: None,
            intermediate_screenshot: None,
            display: Some(0),
            receiver: rx,
            sender: tx,
            crop_info: CropState {
                clicked: false,
                start_ptr: Pos2::default(),
                end_ptr: Pos2::default(),
                curr_ptr: Pos2::default(),
            },
            paint_info: PaintState {
                curr_tool: Tool::None,
                curr_color: [255, 255, 255, 255],
                curr_thickness: 1,
                painting: false,
                last_ptr: Pos2::default(),
                curr_ptr: Pos2::default(),
            },
            action: Action::None,
            timer: Some(0),
            allowed_to_close: true,
            show_confirmation_dialog: false,
            shortcuts: map,
            icons: icons_map,
            tooltips: tooltips_map,
        }
    }
    /// Used to restore state of the application when stopping the crop action for some reason
    fn restore_from_crop(&mut self) {
        self.action = Action::None;
        self.crop_info.reset();
        self.screenshot = self.final_screenshot.clone();
    }

    /// Used to restore state of the screenshot when undoing paint changes
    fn restore_from_paint(&mut self) {
        self.paint_info.reset();
        //Restore the original screenshot
        self.screenshot = self.final_screenshot.clone();
        self.intermediate_screenshot = self.final_screenshot.clone();
    }

    fn save_paint_changes(&mut self) {
        self.paint_info.reset();
        //Save the changed screenshot as final screenshot
        self.final_screenshot = self.screenshot.clone();
    }

    fn copy_image(&mut self) {
        let mut clipboard = Clipboard::new().unwrap();
        let bytes = self.screenshot.as_ref().unwrap().as_bytes();
        let img = arboard::ImageData {
            width: self.screenshot.as_ref().unwrap().width() as usize,
            height: self.screenshot.as_ref().unwrap().height() as usize,
            bytes: Cow::from(bytes),
        };
        let done = clipboard.set_image(img);
    }

    fn render_top_panel(&mut self, ctx: &Context, frame: &mut Frame) {
        TopBottomPanel::top("top panel").show(ctx, |ui| {
            ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                if self.action == Action::None {
                    let screenshot_btn = ui.add(Button::new("Take Screenshot"));
                    let screenshot_save_btn = ui.add(Button::new("Save Screenshot"));
                    let combo_box = ComboBox::from_label("")
                    .width(80.0)
                    .selected_text(format!("🕓 {:?} sec", self.timer.unwrap()))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.timer, Some(0), "🕓 0 sec");
                        ui.selectable_value(&mut self.timer, Some(2), "🕓 2 sec");
                        ui.selectable_value(&mut self.timer, Some(5), "🕓 5 sec");
                        ui.selectable_value(&mut self.timer, Some(10), "🕓 10 sec");
                    });
                    self.display_selector(ui);
                    if screenshot_btn.clicked()
                        || ctx.input_mut(|i| {
                            i.consume_shortcut(
                                self.shortcuts.get(&KeyCommand::TakeScreenshot).unwrap(),
                            )
                        })
                    {
                        self.store_screenshot(frame, ctx);
                    }

                    if screenshot_save_btn.clicked()
                        || ctx.input_mut(|i| {
                            i.consume_shortcut(
                                self.shortcuts.get(&KeyCommand::SaveScreenshot).unwrap(),
                            )
                        })
                    {
                        match &self.screenshot {
                            Some(screenshot) => {
                                save_screenshot(screenshot);
                            }
                            None => {}
                        }
                    }

                    //Spawn paint and crop only if screenshot is already available
                    if self.screenshot.is_some() {
                        let crop_btn = ui.add(Button::new("Crop"));
                        let paint_btn = ui.add(Button::new("Paint"));
                        let copy_btn = ui.add(Button::new("Copy"));
                        if crop_btn.clicked()
                            || ctx.input_mut(|i| {
                                i.consume_shortcut(self.shortcuts.get(&KeyCommand::Crop).unwrap())
                            })
                        {
                            self.action = Action::Crop;
                        }
                        if paint_btn.clicked()
                            || ctx.input_mut(|i| {
                                i.consume_shortcut(self.shortcuts.get(&KeyCommand::Paint).unwrap())
                            })
                        {
                            self.action = Action::Paint;
                        }
                        if copy_btn.clicked() {
                            self.copy_image();
                        }
                    }

                } else if self.action == Action::Crop {
                    self.render_crop_tools(ui);
                } else if self.action == Action::Paint {
                    self.render_paint_tools(ctx, ui);
                }
            })
        });
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
            thread::sleep(Duration::from_millis(timer*1000 + 300));
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
            .selected_text(format!("Display {:?}", self.display.unwrap()))
            .show_ui(ui, |ui| {
                for (i, display) in screen::display_list().into_iter().enumerate() {
                    if ui
                        .selectable_value(
                            &mut self.display.clone().unwrap(),
                            i,
                            format!(
                                "Display {} - {}x{} px",
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

    fn render_central_panel(&mut self, ctx: &Context, frame: &mut Frame) {
        CentralPanel::default().show(ctx, |ui| match self.screenshot.clone() {
            //If screenshot is already available, then show it on the GUI
            Some(screenshot) => {
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
                    if self.action == Action::Crop {
                        self.crop_logic(img);
                    } else if self.action == Action::Paint {
                        self.paint_logic(ctx, img);
                    }
                });
            }
            None => {
                ScrollArea::both().show(ui, |ui| ui.label("No screenshots yet"));
            }
        });
    }

    /// Renders an ImageButton using the svg corresponding to the given name, if the svg failed to load or the name does not correspond to any svg, it spawns a button with the name passed as parameter to icon_button
    fn icon_button(&mut self, name: &str, ctx: &Context, ui: &mut Ui) -> Response {
        match self.icons.get(name) {
            Some(val) => match val {
                Ok(image) => ui
                    .add(ImageButton::new(image.texture_id(ctx), image.size_vec2()))
                    .on_hover_text(self.tooltips.get(name).unwrap_or(&"Error".to_string())),
                Err(_) => ui.add(Button::new(name)),
            },
            None => ui.add(Button::new(name)),
        }
    }

    /// Renders an icon using the svg corresponding to the given name, if the svg failed to load or the name does not correspond to any svg, it spawns a button with the name passed as parameter to icon
    fn icon(&mut self, name: &str, ctx: &Context, ui: &mut Ui) -> Response {
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
        ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
            let save_paint_btn = ui.add(Button::new("Save changes"));
            let undo_paint_btn = ui.add(Button::new("Undo changes"));
            let draw_btn = self.icon_button("pencil-fill", ctx, ui);
            let hollow_rect_btn = self.icon_button("square", ctx, ui);
            let filled_rect_btn = self.icon_button("square-fill", ctx, ui);
            let hollow_circle_btn = self.icon_button("circle", ctx, ui);
            let filled_circle_btn = self.icon_button("circle-fill", ctx, ui);
            let arrow_btn = self.icon_button("arrow-up-right", ctx, ui);
            let highligher_btn = self.icon_button("pen-fill", ctx, ui);
            let eraser_btn = self.icon_button("eraser-fill", ctx, ui);
            let rmv_tool_btn = self.icon_button("x-octagon", ctx, ui);
            ui.color_edit_button_srgba_unmultiplied(&mut self.paint_info.curr_color);
            ui.label("Current tool:");
            let curr_tool = match self.paint_info.curr_tool {
                Tool::Drawing => self.icon("pencil-fill", ctx, ui),
                Tool::HollowRect => self.icon("square", ctx, ui),
                Tool::FilledRect => self.icon("square-fill", ctx, ui),
                Tool::HollowCircle => self.icon("circle", ctx, ui),
                Tool::FilledCircle => self.icon("circle-fill", ctx, ui),
                Tool::Arrow => self.icon("arrow-up-right", ctx, ui),
                Tool::Eraser => self.icon("eraser-fill", ctx, ui),
                Tool::Highlighter => self.icon("pen-fill", ctx, ui),
                Tool::None => ui.add(Label::new("No tool selected")),
            };
            ui.add(Slider::new(&mut self.paint_info.curr_thickness, 0..=30));
            if rmv_tool_btn.clicked() {
                self.paint_info.curr_tool = Tool::None;
            }
            if save_paint_btn.clicked() {
                self.action = Action::None;
                self.save_paint_changes();
            }
            if undo_paint_btn.clicked() {
                self.restore_from_paint();
            }
            if draw_btn.clicked() {
                self.paint_info.curr_tool = Tool::Drawing;
            }
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
            if highligher_btn.clicked(){
                self.paint_info.curr_tool = Tool::Highlighter;
            }
        });
    }

    fn render_crop_tools(&mut self, ui: &mut Ui) {
        let undo_crop_btn = ui.add(Button::new("Stop cropping"));
        if undo_crop_btn.clicked() {
            //To restore image without cropping rect
            self.restore_from_crop();
        }
    }

    /// Logic for cropping the image
    fn crop_logic(&mut self, img: Response) {
        if !self.crop_info.clicked && img.clicked() {
            self.crop_info.start_ptr =
                into_relative_pos(img.interact_pointer_pos().unwrap(), img.rect);
            self.crop_info.clicked = true;
        } else if self.crop_info.clicked && img.clicked() {
            self.crop_info.end_ptr =
                into_relative_pos(img.interact_pointer_pos().unwrap(), img.rect);
            self.crop_info.clicked = false;
            let width = max(
                1,
                (self.crop_info.end_ptr.x - self.crop_info.start_ptr.x).abs() as i32,
            );
            let height = max(
                1,
                (self.crop_info.end_ptr.y - self.crop_info.start_ptr.y).abs() as i32,
            );
            //Permits an easier selection when cropping, allowing to generate the crop area in all directions
            let mut start_ptr = self.crop_info.start_ptr;
            if self.crop_info.curr_ptr.x < self.crop_info.start_ptr.x {
                start_ptr.x = self.crop_info.curr_ptr.x;
            }
            if self.crop_info.curr_ptr.y < self.crop_info.start_ptr.y {
                start_ptr.y = self.crop_info.curr_ptr.y;
            }
            let new_screen = self.final_screenshot.as_ref().unwrap().crop_imm(
                start_ptr.x as u32,
                start_ptr.y as u32,
                width as u32,
                height as u32,
            );
            self.screenshot = Some(DynamicImage::from(new_screen.clone()));
            self.intermediate_screenshot = Some(DynamicImage::from(new_screen.clone()));
            self.final_screenshot = Some(DynamicImage::from(new_screen));
            self.action = Action::None;
        }
        if self.crop_info.clicked && img.secondary_clicked() {
            self.restore_from_crop();
        }
        if self.crop_info.clicked {
            let white = Rgb([255u8, 255u8, 255u8]);
            self.crop_info.curr_ptr = match img.hover_pos() {
                Some(pos) => into_relative_pos(pos, img.rect),
                None => self.crop_info.curr_ptr,
            };
            let width = max(
                1,
                (self.crop_info.curr_ptr.x - self.crop_info.start_ptr.x).abs() as i32,
            );
            let height = max(
                1,
                (self.crop_info.curr_ptr.y - self.crop_info.start_ptr.y).abs() as i32,
            );
            //Permits an easier selection when cropping, allowing to generate the crop area in all directions
            let mut start_ptr = self.crop_info.start_ptr;
            if self.crop_info.curr_ptr.x < self.crop_info.start_ptr.x {
                start_ptr.x = self.crop_info.curr_ptr.x;
            }
            if self.crop_info.curr_ptr.y < self.crop_info.start_ptr.y {
                start_ptr.y = self.crop_info.curr_ptr.y;
            }
            let new_screen: Image<Rgb<u8>> = drawing::draw_hollow_rect(
                self.final_screenshot.as_ref().unwrap().as_rgb8().unwrap(),
                imageproc::rect::Rect::at(start_ptr.x as i32, start_ptr.y as i32)
                    .of_size(width as u32, height as u32),
                white,
            );
            self.screenshot = Some(DynamicImage::from(new_screen));
        }
    }

    /// Logic for painting on the image
    fn paint_logic(&mut self, ctx: &Context, img: Response) {
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
            let new_screen = self.paint_info.draw_shape(self.intermediate_screenshot.as_ref().unwrap(), self.final_screenshot.as_ref().unwrap());
            if self.paint_info.curr_tool == Tool::Drawing || self.paint_info.curr_tool == Tool::Highlighter{
                self.paint_info.last_ptr = self.paint_info.curr_ptr;
                self.intermediate_screenshot = Some(DynamicImage::from(new_screen.clone()));
            }
            self.screenshot = Some(DynamicImage::from(new_screen));
        } else if img.drag_released() {
            self.intermediate_screenshot = Some(DynamicImage::from(
                self.screenshot.as_ref().unwrap().clone(),
            ));
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
            .add_filter("WEBP", &["WEBP"]) //ToDelete?
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
                image::ColorType::Rgb8,
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
                self.screenshot = Some(screenshot.clone());
                self.intermediate_screenshot = Some(screenshot.clone());
                self.final_screenshot = Some(screenshot);
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
