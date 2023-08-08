use crate::screen::{self, display_list, take_screenshot};
use eframe::egui::{
    Align, Button, CentralPanel, ColorImage, ComboBox, Context, Layout, ScrollArea, TopBottomPanel,
};
use eframe::{run_native, NativeOptions};
use eframe::{App, Frame};
use egui_extras::RetainedImage;
use image::{EncodableLayout, ImageBuffer, Rgb};
use scrap::Display;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

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

struct RustShot {
    screenshot: Option<ImageBuffer<Rgb<u8>, Vec<u8>>>,
    display: Option<usize>,
    receiver: Receiver<ImageBuffer<Rgb<u8>, Vec<u8>>>,
    sender: Sender<ImageBuffer<Rgb<u8>, Vec<u8>>>,
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
            display: Some(0),
            receiver: rx,
            sender: tx,
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
                self.screenshot = Some(screenshot);
            }
            Err(err) => (),
        }
        TopBottomPanel::top("top panel").show(ctx, |ui| {
            ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                let screenshot_btn = ui.add(Button::new("Take Screenshot"));
                if screenshot_btn.clicked() {
                    //Hide the application window
                    frame.set_visible(false);
                    let tx = self.sender.clone();
                    let c = ctx.clone();
                    let value = Arc::new(self.display.unwrap().clone());
                    println!("Display : {}", value);
                    //Thread that manages screenshots
                    //TODO : Find a way to share the selected display with the thread
                    thread::spawn(move || {
                        thread::sleep(Duration::from_millis(300));
                        let current_display = select_display(*value as usize).expect("Cannot select the correct display");
                        let screenshot = take_screenshot(current_display).unwrap();
                        match tx.send(screenshot) {
                            //Force update() to be called again, so that the application window is made visible again. (when it's not visible otherwise update won't be called)
                            Ok(_) => c.request_repaint(),
                            Err(err) => println!("{}", err),
                        }
                    });
                }
                let screenshot_save_btn = ui.add(Button::new("Save Screenshot"));
                if screenshot_save_btn.clicked() {
                    match &self.screenshot {
                        Some(screenshot) => {
                            let path =
                                tinyfiledialogs::save_file_dialog("Select save location", "./");
                            match path {
                                Some(path) => {
                                    match image::save_buffer(
                                        path,
                                        &screenshot,
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
                let mut value = 0;
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
                    });

                let crop_btn = ui.add(Button::new("Crop"));
            })
        });
        CentralPanel::default().show(ctx, |ui| match &mut self.screenshot {
            Some(screenshot) => ScrollArea::both().show(ui, |ui| {
                RetainedImage::from_color_image(
                    "screenshot",
                    ColorImage::from_rgb(
                        [screenshot.width() as usize, screenshot.height() as usize],
                        screenshot.as_bytes(),
                    ),
                )
                .show(ui)
            }),
            None => ScrollArea::both().show(ui, |ui| ui.label("No screenshots yet")),
        });
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
