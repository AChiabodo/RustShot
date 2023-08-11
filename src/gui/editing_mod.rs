
use crate::gui::image_proc_extra_mod::*;
use std::cmp::max;
use eframe::egui::Pos2;
use image::DynamicImage;
use imageproc::drawing;

#[derive(PartialEq, Eq)]
pub enum Tool {
    Drawing,
    HollowRect,
    FilledRect,
    Arrow,
    HollowCircle,
    FilledCircle,
    Eraser,
    Highlighter,
    None,
}

pub struct PaintState {
    pub curr_tool: Tool,
    pub curr_color: [u8; 4],
    pub curr_thickness: usize,
    pub painting: bool,
    pub last_ptr: Pos2,
    pub curr_ptr: Pos2,
}

impl PaintState {
    /// Reset the paint state to its default values, including the current tool and color
    pub fn reset(&mut self) {
        self.painting = false;
        self.last_ptr = Pos2::default();
        self.curr_ptr = Pos2::default();
        self.curr_tool = Tool::None;
        self.curr_color = [255, 255, 255, 255];
    }

    /// Reset the paint state to its default values, excluding the current tool and color
    pub fn soft_reset(&mut self) {
        self.painting = false;
        self.last_ptr = Pos2::default();
        self.curr_ptr = Pos2::default();
    }

    ///Draw a shape on the given img based on the field inside [self] ([curr_tool], [curr_color], [last_ptr], [curr_ptr])
    pub fn draw_shape(&self, img: &DynamicImage, original_img:&DynamicImage) -> DynamicImage {
        let mut start_ptr = self.last_ptr;
        let width = max(1, (self.curr_ptr.x - self.last_ptr.x).abs() as i32);
        let height = max(1, (self.curr_ptr.y - self.last_ptr.y).abs() as i32);
        if self.curr_tool != Tool::Drawing && self.curr_tool != Tool::FilledCircle && self.curr_tool != Tool::HollowCircle {
            //Permits an easier selection, allowing to generate the area in all directions
            if self.curr_ptr.x < self.last_ptr.x {
                start_ptr.x = self.curr_ptr.x;
            }
            if self.curr_ptr.y < self.last_ptr.y {
                start_ptr.y = self.curr_ptr.y;
            }
        }
        let mut new_screen = img.clone();
        match self.curr_tool {
            Tool::Drawing => {
                new_screen = draw_thick_line(img, (self.last_ptr.x, self.last_ptr.y), (self.curr_ptr.x, self.curr_ptr.y), self.curr_thickness, self.curr_color.into());
            }
            Tool::HollowRect => {
                drawing::draw_hollow_rect_mut(&mut new_screen, imageproc::rect::Rect::at(start_ptr.x as i32, start_ptr.y as i32).of_size(width as u32, height as u32), self.curr_color.into());
            }
            Tool::FilledRect => {
                drawing::draw_filled_rect_mut(&mut new_screen, imageproc::rect::Rect::at(start_ptr.x as i32, start_ptr.y as i32).of_size(width as u32, height as u32), self.curr_color.into());
            }
            Tool::HollowCircle => {
                let radius = ((width.pow(2) + height.pow(2)) as f64).sqrt() as i32;
                drawing::draw_hollow_circle_mut(&mut new_screen, (start_ptr.x as i32, start_ptr.y as i32), radius, self.curr_color.into());
            }
            Tool::FilledCircle => {
                let radius = ((width.pow(2) + height.pow(2)) as f64).sqrt() as i32;
                drawing::draw_filled_circle_mut(&mut new_screen, (start_ptr.x as i32, start_ptr.y as i32), radius, self.curr_color.into());
            }
            Tool::Arrow => {
                drawing::draw_line_segment_mut(&mut new_screen, (start_ptr.x, start_ptr.y), (self.curr_ptr.x, self.curr_ptr.y), self.curr_color.into());
            }
            Tool::Highlighter => {
                new_screen = highlight_line(original_img, img, (self.last_ptr.x, self.last_ptr.y), (self.curr_ptr.x, self.curr_ptr.y), self.curr_thickness, self.curr_color.into());
            }
            _ => {}
        }
        return new_screen;
    }
}

pub struct CropState {
    pub clicked: bool,
    pub start_ptr: Pos2,
    pub end_ptr: Pos2,
    pub curr_ptr: Pos2,
}

impl CropState {
    /// Reset the crop state to its default values
    pub fn reset(&mut self) {
        self.clicked = false;
        self.start_ptr = Pos2::default();
        self.end_ptr = Pos2::default();
        self.curr_ptr = Pos2::default();
    }
}