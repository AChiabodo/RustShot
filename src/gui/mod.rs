use std::cmp::max;
use crate::screen::{self, display_list, take_screenshot};
use eframe::egui::{Align, Button, CentralPanel, ColorImage, ComboBox, Context, ImageButton, ImageData, Layout, Pos2, Rect, Response, ScrollArea, Sense, Shape, TopBottomPanel};
use eframe::{run_native, NativeOptions};
use eframe::{App, Frame};
use egui_extras::RetainedImage;
use image::{DynamicImage, EncodableLayout, GenericImage, ImageBuffer, Rgb, Rgba, RgbImage};
use scrap::Display;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use std::time::Duration;
use eframe::egui::accesskit::Size;
use eframe::egui::ImageData::Color;
use imageproc::definitions::Image;
use imageproc::drawing;
use imageproc::drawing::Canvas;
use imageproc::point::Point;


fn select_display(index: usize) -> Option<Display> {
    let mut iter = screen::display_list().into_iter().enumerate();
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
    };
}

/// Transform the absolute position ([Pos2]) of the mouse on the application window into a relative position with respect to the given [Rect]
///
/// [Rect] must be meaningful with respect to the application window. (It needs to actually be a part of the application window to obtain a meaningful relative position)
fn into_relative_pos(pos: Pos2, rect: Rect) -> Pos2 {
    Pos2::new(pos.x - rect.left(), pos.y - rect.top())
}

#[derive(PartialEq, Eq)]
enum Action {
    Crop,
    Paint,
    None,
}

#[derive(PartialEq, Eq)]
enum Tool {
    Drawing,
    HollowRect,
    FilledRect,
    Arrow,
    HollowCircle,
    FilledCircle,
    None,
}

struct PaintState {
    curr_tool: Tool,
    curr_color: Rgb<u8>,
    painting: bool,
    last_ptr: Pos2,
    curr_ptr: Pos2,
}

impl PaintState {
    /// Reset the paint state to its default values
    fn reset(&mut self) {
        self.painting = false;
        self.last_ptr = Pos2::default();
        self.curr_ptr = Pos2::default();
    }

    fn draw_shape(&self, img: &RgbImage) -> Option<Image<Rgb<u8>>> {
        let mut start_ptr = self.last_ptr;
        let width = max(1, (self.curr_ptr.x - self.last_ptr.x).abs() as i32);
        let height = max(1, (self.curr_ptr.y - self.last_ptr.y).abs() as i32);
        if self.curr_tool != Tool::Drawing || self.curr_tool != Tool::FilledCircle || self.curr_tool != Tool::HollowCircle {
            //Permits an easier selection, allowing to generate the area in all directions
            if self.curr_ptr.x < self.last_ptr.x {
                start_ptr.x = self.curr_ptr.x;
            }
            if self.curr_ptr.y < self.last_ptr.y {
                start_ptr.y = self.curr_ptr.y;
            }
        }
        let mut new_screen = None;
        match self.curr_tool {
            Tool::Drawing => {
                new_screen = Some(drawing::draw_line_segment(img, (start_ptr.x, start_ptr.y), (self.curr_ptr.x, self.curr_ptr.y), self.curr_color));
            }
            Tool::HollowRect => {
                new_screen = Some(drawing::draw_hollow_rect(img,imageproc::rect::Rect::at(start_ptr.x as i32, start_ptr.y as i32).of_size(width as u32, height as u32), self.curr_color));
            }
            Tool::FilledRect => {
                new_screen = Some(drawing::draw_filled_rect(img,imageproc::rect::Rect::at(start_ptr.x as i32, start_ptr.y as i32).of_size(width as u32, height as u32), self.curr_color));
            }
            Tool::HollowCircle => {
                let radius = ((width.pow(2) + height.pow(2)) as f64).sqrt() as i32;
                new_screen = Some(drawing::draw_hollow_circle(img, (start_ptr.x as i32, start_ptr.y as i32), radius, self.curr_color));
            }
            Tool::FilledCircle => {
                let radius = ((width.pow(2) + height.pow(2)) as f64).sqrt() as i32;
                new_screen = Some(drawing::draw_filled_circle(img, (start_ptr.x as i32, start_ptr.y as i32), radius, self.curr_color));
            }
            Tool::Arrow => {
                new_screen = Some(drawing::draw_line_segment(img, (start_ptr.x, start_ptr.y), (self.curr_ptr.x, self.curr_ptr.y), self.curr_color));
            }
            _ => {}
        }
        return new_screen;
    }
}


struct CropState {
    clicked: bool,
    start_ptr: Pos2,
    end_ptr: Pos2,
    curr_ptr: Pos2,
}

impl CropState {
    /// Reset the crop state to its default values
    fn reset(&mut self) {
        self.clicked = false;
        self.start_ptr = Pos2::default();
        self.end_ptr = Pos2::default();
        self.curr_ptr = Pos2::default();
    }
}

struct RustShot {
    screenshot: Option<DynamicImage>,
    final_screenshot: Option<DynamicImage>,
    display: Option<usize>,
    receiver: Receiver<DynamicImage>,
    sender: Sender<DynamicImage>,
    crop_info: CropState,
    paint_info: PaintState,
    action: Action,
}

impl RustShot {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.
        let (tx, rx) = channel();
        RustShot {
            screenshot: None,
            final_screenshot: None,
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
                curr_color: Rgb([255u8, 255u8, 255u8]),
                painting: false,
                last_ptr: Pos2::default(),
                curr_ptr: Pos2::default(),
            },
            action: Action::None,
        }
    }
    /// Used to restore state of the application when stopping the crop action for some reason
    fn restore_from_crop(&mut self) {
        self.action = Action::None;
        self.crop_info.reset();
        self.screenshot = self.final_screenshot.clone();
    }

    /// Used to restore state of the application when stopping the paint action for some reason
    fn restore_from_paint(&mut self) {
        self.paint_info.reset();
        //Restore the original screenshot
        self.screenshot = self.final_screenshot.clone()
    }

    fn save_paint_changes(&mut self) {
        self.paint_info.reset();
        //Save the changed screenshot as final screenshot
        self.final_screenshot = self.screenshot.clone();
    }

    fn render_top_panel(&mut self, ctx: &Context, frame: &mut Frame) {
        TopBottomPanel::top("top panel").show(ctx, |ui| {
            ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                if self.action == Action::None {
                    let screenshot_btn = ui.add(Button::new("Take Screenshot"));
                    let screenshot_save_btn = ui.add(Button::new("Save Screenshot"));
                    let crop_btn = ui.add(Button::new("Crop"));
                    let paint_btn = ui.add(Button::new("Paint"));
                    if screenshot_btn.clicked() {
                        //Hide the application window
                        frame.set_visible(false);
                        let tx = self.sender.clone();
                        let c = ctx.clone();
                        let value = self.display.unwrap().clone();
                        println!("Display : {}", value);
                        //Thread that manages screenshots
                        thread::spawn(move || {
                            thread::sleep(Duration::from_millis(300));
                            let current_display = select_display(value as usize).expect("Cannot select the correct display");
                            let screenshot = take_screenshot(current_display).unwrap();
                            match tx.send(screenshot) {
                                //Force update() to be called again, so that the application window is made visible again. (when it's not visible otherwise update won't be called)
                                Ok(_) => c.request_repaint(),
                                Err(err) => println!("{}", err),
                            }
                        });
                    }
                    if screenshot_save_btn.clicked() {
                        match &self.screenshot {
                            Some(screenshot) => {
                                let path =
                                    tinyfiledialogs::save_file_dialog("Select save location", "./");
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
                            None => {}
                        }
                    }
                    if crop_btn.clicked() {
                        self.action = Action::Crop;
                    }
                    if paint_btn.clicked() {
                        self.action = Action::Paint;
                    }
                }
                /*let mut value = 0;
                let mut selected = 0;
                let display_selector = ComboBox::from_label("Select Display")
                    .selected_text(format!("{:?}", self.display.unwrap()))
                    .show_ui(ui, |ui| {
                        for (i, display) in screen::display_list().into_iter().enumerate() {
                            if ui
                                .selectable_value(
                                    &mut self.display.clone().unwrap(),
                                    i,
                                    format!(
                                        "Display {} - {}x{} px",
                                        i,
                                        display.width(),
                                        display.height()
                                    ),
                                )
                                .clicked()
                            {
                                selected = i;
                                println!("Selected : {}", selected);
                                self.display = Some(selected);
                            }
                        }
                    });*/
                else if self.action == Action::Crop {
                    let undo_crop_btn = ui.add(Button::new("Stop cropping"));
                    if undo_crop_btn.clicked() {
                        //To restore image without cropping rect
                        self.restore_from_crop();
                    }
                } else if self.action == Action::Paint {
                    let save_paint_btn = ui.add(Button::new("Save changes"));
                    let undo_paint_btn = ui.add(Button::new("Undo changes"));
                    if save_paint_btn.clicked() {
                        self.action = Action::None;
                        self.save_paint_changes();
                    }
                    if undo_paint_btn.clicked(){
                        self.action = Action::None;
                        self.restore_from_paint();
                    }
                }
            })
        });
    }

    fn render_central_panel(&mut self, ctx: &Context, frame: &mut Frame) {
        CentralPanel::default().show(ctx, |ui| match self.screenshot.clone() {
            //If screenshot is already available, then show it on the GUI
            Some(screenshot) => {
                ScrollArea::both().show(ui, |ui| {
                    let retained_img = RetainedImage::from_color_image(
                        "screenshot",
                        ColorImage::from_rgb(
                            [screenshot.width() as usize, screenshot.height() as usize],
                            screenshot.as_bytes(),
                        ),
                    );
                    let img = ui.add(ImageButton::new(retained_img.texture_id(ctx), retained_img.size_vec2()).frame(false).sense(Sense::click_and_drag()));
                    if self.action == Action::Crop {
                        self.crop_logic(img);
                    } else if self.action == Action::Paint {
                        self.paint_logic(img);
                    }
                });
            }
            None => { ScrollArea::both().show(ui, |ui| ui.label("No screenshots yet")); }
        });
    }

    /// Logic for cropping the image
    fn crop_logic(&mut self, img: Response) {
        if !self.crop_info.clicked && img.clicked() {
            self.crop_info.start_ptr = into_relative_pos(img.interact_pointer_pos().unwrap(), img.rect);
            self.crop_info.clicked = true;
        } else if self.crop_info.clicked && img.clicked() {
            self.crop_info.end_ptr = into_relative_pos(img.interact_pointer_pos().unwrap(), img.rect);
            self.crop_info.clicked = false;
            let width = max(1, (self.crop_info.end_ptr.x - self.crop_info.start_ptr.x).abs() as i32);
            let height = max(1, (self.crop_info.end_ptr.y - self.crop_info.start_ptr.y).abs() as i32);
            //Permits an easier selection when cropping, allowing to generate the crop area in all directions
            let mut start_ptr = self.crop_info.start_ptr;
            if self.crop_info.curr_ptr.x < self.crop_info.start_ptr.x {
                start_ptr.x = self.crop_info.curr_ptr.x;
            }
            if self.crop_info.curr_ptr.y < self.crop_info.start_ptr.y {
                start_ptr.y = self.crop_info.curr_ptr.y;
            }
            let new_screen = self.final_screenshot.as_ref().unwrap().crop_imm(start_ptr.x as u32, start_ptr.y as u32, width as u32, height as u32);
            self.screenshot = Some(DynamicImage::from(new_screen.clone()));
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
            let width = max(1, (self.crop_info.curr_ptr.x - self.crop_info.start_ptr.x).abs() as i32);
            let height = max(1, (self.crop_info.curr_ptr.y - self.crop_info.start_ptr.y).abs() as i32);
            //Permits an easier selection when cropping, allowing to generate the crop area in all directions
            let mut start_ptr = self.crop_info.start_ptr;
            if self.crop_info.curr_ptr.x < self.crop_info.start_ptr.x {
                start_ptr.x = self.crop_info.curr_ptr.x;
            }
            if self.crop_info.curr_ptr.y < self.crop_info.start_ptr.y {
                start_ptr.y = self.crop_info.curr_ptr.y;
            }
            let new_screen: Image<Rgb<u8>> = drawing::draw_hollow_rect(self.final_screenshot.as_ref().unwrap().as_rgb8().unwrap(),
                                                                       imageproc::rect::Rect::at(start_ptr.x as i32, start_ptr.y as i32).of_size(width as u32, height as u32), white);
            self.screenshot = Some(DynamicImage::from(new_screen));
        }
    }

    /// Logic for painting on the image
    fn paint_logic(&mut self, img: Response) {
        if img.dragged() {
            if !self.paint_info.painting{
                self.paint_info.painting = true;
                self.paint_info.last_ptr = into_relative_pos(img.interact_pointer_pos().unwrap(), img.rect);
            }
            self.paint_info.curr_ptr = match img.hover_pos() {
                Some(pos) => into_relative_pos(pos, img.rect),
                None => self.paint_info.last_ptr,
            };
            let new_screen = match self.paint_info.draw_shape(self.screenshot.as_ref().unwrap().as_rgb8().unwrap()) {
                Some(screen) => screen,
                None => self.screenshot.as_ref().unwrap().as_rgb8().unwrap().clone(),
            };
            self.paint_info.last_ptr = self.paint_info.curr_ptr;
            self.screenshot = Some(DynamicImage::from(new_screen));
        } else if img.drag_released(){
            self.paint_info.reset();
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
                self.screenshot = Some(screenshot.clone());
                self.final_screenshot = Some(screenshot);
            }
            Err(err) => (),
        }
        self.render_top_panel(ctx, frame);
        self.render_central_panel(ctx, frame);
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
