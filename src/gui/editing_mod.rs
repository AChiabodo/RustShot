
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
    pub fn apply_tool(&self, img: &mut Image) {
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
        match self.curr_tool {
            Tool::Drawing => {
                draw_thick_line(&mut img.image, (self.last_ptr.x, self.last_ptr.y), (self.curr_ptr.x, self.curr_ptr.y), self.curr_thickness, self.curr_color.into());
            }
            Tool::HollowRect => {
                draw_thick_hollow_rect_mut(&mut img.image, imageproc::rect::Rect::at(start_ptr.x as i32, start_ptr.y as i32).of_size(width as u32, height as u32), self.curr_thickness, self.curr_color.into());
            }
            Tool::FilledRect => {
                drawing::draw_filled_rect_mut(&mut img.image, imageproc::rect::Rect::at(start_ptr.x as i32, start_ptr.y as i32).of_size(width as u32, height as u32), self.curr_color.into());
            }
            Tool::HollowCircle => {
                let radius = ((width.pow(2) + height.pow(2)) as f64).sqrt() as i32;
                draw_thick_hollow_circle_mut(&mut img.image, (start_ptr.x as i32, start_ptr.y as i32), radius, self.curr_thickness, self.curr_color.into());
            }
            Tool::FilledCircle => {
                let radius = ((width.pow(2) + height.pow(2)) as f64).sqrt() as i32;
                drawing::draw_filled_circle_mut(&mut img.image, (start_ptr.x as i32, start_ptr.y as i32), radius, self.curr_color.into());
            }
            Tool::Arrow => {
                draw_arrow(&mut img.image, (self.last_ptr.x, self.last_ptr.y), (self.curr_ptr.x, self.curr_ptr.y), self.curr_thickness, self.curr_color.into());
            }
            Tool::Highlighter => {
                highlight_line(&img.eraser_image, &mut img.image, (self.last_ptr.x, self.last_ptr.y), (self.curr_ptr.x, self.curr_ptr.y), self.curr_thickness, self.curr_color.into());
            }
            Tool::Crop => {
                drawing::draw_hollow_rect_mut(&mut img.image, imageproc::rect::Rect::at(start_ptr.x as i32, start_ptr.y as i32).of_size(width as u32, height as u32), self.curr_color.into());
            }
            Tool::Eraser => {
                erase_thick_line(&img.eraser_image, &mut img.image, (self.last_ptr.x, self.last_ptr.y), (self.curr_ptr.x, self.curr_ptr.y), self.curr_thickness);
            }
            _ => {}
        }
    }
}


#[derive(Clone)]
pub struct Image {
    image: DynamicImage,
    eraser_image: DynamicImage,
}

impl Image {
    pub fn new(image:DynamicImage, eraser_image: DynamicImage) -> Self {
        Image { image, eraser_image}
    }
    pub fn get_image(&self) -> DynamicImage {
        self.image.clone()
    }

    pub fn get_eraser_image(&self) -> DynamicImage {
        self.eraser_image.clone()
    }
}

pub struct ImageStack {
    images: VecDeque<Image>,
    redo_images: VecDeque<Image>,
    tmp_image: Image,
    final_image: Image,
    curr_last_crop: Image,
}

impl ImageStack {
    pub fn new(image: DynamicImage)-> Self{
        let mut images = VecDeque::new();
        images.push_front(Image{image: image.clone(), eraser_image: image.clone()});
        ImageStack {
            images,
            redo_images: VecDeque::new(),
            tmp_image: Image{image: image.clone(), eraser_image: image.clone()},
            final_image: Image{image: image.clone(), eraser_image: image.clone()},
            curr_last_crop: Image{image: image.clone(), eraser_image: image.clone()},
        }
    }

    /// Push a new image to the redo_images stack
    pub fn push_redo_image(&mut self, image: Image) {
        self.redo_images.push_front(image);
    }

    /// Push an image from the redo_images stack
    pub fn pop_redo_image(&mut self) -> Option<Image> {
        self.redo_images.pop_front()
    }

    pub fn get_images_len(&self) -> usize {
        self.images.len()
    }

    pub fn get_redo_images_len(&self) -> usize {
        self.redo_images.len()
    }

    /// Pop the last stacked image in the image stack, removing it. Returns [final image] if the stack is empty
    pub fn pop_last_image(&mut self) -> Image {
        match self.images.pop_front(){
            Some(img) => img,
            None => self.final_image.clone()
        }
    }

    ///Get the width of the final image
    pub fn get_width (&self) -> usize {
        return self.final_image.image.width() as usize;
    }

    ///Get the height of the final image
    pub fn get_height (&self) -> usize {
        return self.final_image.image.height() as usize;
    }

    /// Get the last stacked image in the image stack, without removing it. Returns [final image] if the stack is empty
    pub fn get_last_image(&self) -> Image {
        match self.images.front(){
            Some(img) => (*img).clone(),
            None => self.final_image.clone()
        }
    }

    /// Get the first stacked image in the image stack, without removing it. Returns [final image] if the stack is empty
    pub fn get_first_image (&self) -> Image {
        match self.images.back(){
            Some(img) => (*img).clone(),
            None => self.final_image.clone()
        }
    }

    pub fn get_tmp_image (&self) -> Image {
        return self.tmp_image.clone();
    }

    pub fn set_tmp_image (&mut self, image: Image) {
        self.tmp_image = image.clone();
    }

    /// Stack an image in the image stack
    pub fn stack_image(&mut self, image: Image) {
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
        self.redo_images.clear();
        self.stack_image(self.final_image.clone());
    }

    /// Get the final image
    pub fn get_final_image (&self) -> Image {
        self.final_image.clone()
    }

    pub fn set_final_image(&mut self, image: Image) {
        self.final_image = image;
    }


}