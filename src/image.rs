use eframe::{
	egui::{TextureId, Vec2},
	epi,
};
use image::{DynamicImage, GenericImageView};

pub struct ImageDetails {
	pub id: TextureId,
	pub size: Vec2,
	frame: epi::Frame,
}

// A lot taken from the egui example:
// https://github.com/emilk/egui/blob/master/eframe/examples/image.rs
pub fn image_to_epi_format(image: DynamicImage) -> (Vec2, epi::Image) {
	let image_buffer = image.to_rgba8();
	let size = [image.width() as usize, image.height() as usize];
	let pixels = image_buffer.into_vec();
	(
		Vec2::new(size[0] as f32, size[1] as f32),
		epi::Image::from_rgba_unmultiplied(size, &pixels),
	)
}

impl ImageDetails {
	pub fn new(frame: epi::Frame, size: Vec2, image: epi::Image) -> Self {
		let id = frame.alloc_texture(image);
		ImageDetails { id, frame, size }
	}

	pub fn from_image(frame: epi::Frame, image: DynamicImage) -> Self {
		let (size, image) = image_to_epi_format(image);
		Self::new(frame, size, image)
	}
}

impl Drop for ImageDetails {
	fn drop(&mut self) {
		self.frame.free_texture(self.id);
	}
}
