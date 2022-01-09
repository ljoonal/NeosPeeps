use eframe::{
	egui::{TextureId, Vec2},
	epi,
};
use image::{DynamicImage, GenericImageView};

pub struct TextureDetails {
	pub id: TextureId,
	pub size: Vec2,
	frame: epi::Frame,
}

// A lot taken from the egui example:
// https://github.com/emilk/egui/blob/master/eframe/examples/image.rs
pub fn to_epi_format(image: &DynamicImage) -> (Vec2, epi::Image) {
	let image_buffer = image.to_rgba8();
	let size = [image.width(), image.height()];
	let pixels = image_buffer.into_vec();
	(
		#[allow(clippy::cast_precision_loss)]
		Vec2::new(size[0] as f32, size[1] as f32),
		epi::Image::from_rgba_unmultiplied(
			[size[0] as usize, size[1] as usize],
			&pixels,
		),
	)
}

impl TextureDetails {
	pub fn new(frame: epi::Frame, size: Vec2, image: epi::Image) -> Self {
		let id = frame.alloc_texture(image);
		Self { id, size, frame }
	}

	pub fn from_image(frame: epi::Frame, image: &DynamicImage) -> Self {
		let (size, image) = to_epi_format(image);
		Self::new(frame, size, image)
	}
}

impl Drop for TextureDetails {
	fn drop(&mut self) {
		self.frame.free_texture(self.id);
	}
}
