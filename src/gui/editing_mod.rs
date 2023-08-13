
use crate::gui::image_proc_extra_mod::*;
use std::cmp::max;
use std::collections::VecDeque;
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
    Crop,
    Highlighter,
    None,
}

enum Shape {
    Pencil { points: Vec<(usize, usize)>, thickness: usize },
    Highlighter { points: Vec<(usize, usize)>, thickness: usize },
    HollowRect { point: (usize, usize), width: usize, height: usize},
    FilledRect { point: (usize, usize), width: usize, height: usize},
    HollowCircle { center: (usize, usize), radius: usize},
    FilledCircle { center: (usize, usize), radius: usize},
}

pub struct DrawObject {
    stack_index: usize,
    shape: Shape,
}


pub struct PaintState {
    pub curr_tool: Tool,
    pub curr_color: [u8; 4],
    pub curr_thickness: usize,
    pub painting: bool,
    pub last_ptr: Pos2,
    pub curr_ptr: Pos2,
    pub drawn_objects: Vec<DrawObject>,
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
    pub fn draw_shape(&self, img: &mut DynamicImage, original_img:&DynamicImage) {
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
                draw_thick_line(img, (self.last_ptr.x, self.last_ptr.y), (self.curr_ptr.x, self.curr_ptr.y), self.curr_thickness, self.curr_color.into());
            }
            Tool::HollowRect => {
                drawing::draw_hollow_rect_mut(img, imageproc::rect::Rect::at(start_ptr.x as i32, start_ptr.y as i32).of_size(width as u32, height as u32), self.curr_color.into());
            }
            Tool::FilledRect => {
                drawing::draw_filled_rect_mut(img, imageproc::rect::Rect::at(start_ptr.x as i32, start_ptr.y as i32).of_size(width as u32, height as u32), self.curr_color.into());
            }
            Tool::HollowCircle => {
                let radius = ((width.pow(2) + height.pow(2)) as f64).sqrt() as i32;
                drawing::draw_hollow_circle_mut(img, (start_ptr.x as i32, start_ptr.y as i32), radius, self.curr_color.into());
            }
            Tool::FilledCircle => {
                let radius = ((width.pow(2) + height.pow(2)) as f64).sqrt() as i32;
                drawing::draw_filled_circle_mut(img, (start_ptr.x as i32, start_ptr.y as i32), radius, self.curr_color.into());
            }
            Tool::Arrow => {
                draw_arrow(img, (self.last_ptr.x, self.last_ptr.y), (self.curr_ptr.x, self.curr_ptr.y), self.curr_thickness, self.curr_color.into());
            }
            Tool::Highlighter => {
                highlight_line(original_img, img, (self.last_ptr.x, self.last_ptr.y), (self.curr_ptr.x, self.curr_ptr.y), self.curr_thickness, self.curr_color.into());
            }
            Tool::Crop => {
                drawing::draw_hollow_rect_mut(img, imageproc::rect::Rect::at(start_ptr.x as i32, start_ptr.y as i32).of_size(width as u32, height as u32), self.curr_color.into());
            }
            Tool::Eraser => {
                erase_thick_line(original_img, img, (self.last_ptr.x, self.last_ptr.y), (self.curr_ptr.x, self.curr_ptr.y), self.curr_thickness);
            }
            _ => {}
        }
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

pub struct ImageStack {
    pub images: VecDeque<DynamicImage>,
    pub tmp_image: DynamicImage,
    pub final_image: DynamicImage,
}

impl ImageStack {
    pub fn new(image: DynamicImage)-> Self{
        let mut images = VecDeque::new();
        images.push_front(image.clone());
        ImageStack {
            images,
            tmp_image: image.clone(),
            final_image: image.clone(),
        }
    }

    /// Pop the last stacked image in the image stack, removing it. Returns [final image] if the stack is empty
    pub fn pop_last_image(&mut self) -> DynamicImage {
        match self.images.pop_front(){
            Some(img) => img,
            None => self.final_image.clone()
        }
    }

    ///Get the width of the final image
    pub fn get_width (&self) -> usize {
        return self.final_image.width() as usize;
    }

    ///Get the height of the final image
    pub fn get_height (&self) -> usize {
        return self.final_image.height() as usize;
    }

    /// Get the last stacked image in the image stack, without removing it. Returns [final image] if the stack is empty
    pub fn get_last_image(&self) -> DynamicImage {
        match self.images.front(){
            Some(img) => img.clone(),
            None => self.final_image.clone()
        }
    }

    /// Get the first stacked image in the image stack, without removing it. Returns [final image] if the stack is empty
    pub fn get_first_image (&self) -> DynamicImage {
        match self.images.back(){
            Some(img) => img.clone(),
            None => self.final_image.clone()
        }
    }

    pub fn get_tmp_image (&self) -> DynamicImage {
        return self.tmp_image.clone();
    }

    pub fn set_tmp_image (&mut self, image: DynamicImage) {
        self.tmp_image = image.clone();
    }

    /// Stack an image in the image stack
    pub fn stack_image(&mut self, image: DynamicImage) {
        self.images.push_front(image);
    }

    /// Restore the image_stack to the first stacked image, or to [final image] if the stack is empty
    pub fn restore(&mut self) {
        let img = self.images.pop_back().unwrap_or(self.final_image.clone());
        self.images.clear();
        self.images.push_front(img.clone());
        self.tmp_image = img.clone();
        self.final_image = img.clone();
    }
    /// Clear the image stack
    pub fn clear_stack(&mut self) {
        self.images.clear();
    }

    /// Save all changes made based on the current last image in the stack
    pub fn save_changes (&mut self) {
        self.final_image = self.get_last_image();
        self.tmp_image = self.get_last_image();
        self.images.clear();
        self.stack_image(self.final_image.clone());
    }

    /// Get the final image
    pub fn get_final_image (&self) -> DynamicImage {
        self.final_image.clone()
    }

    pub fn set_final_image(&mut self, image: DynamicImage) {
        self.final_image = image;
    }


}