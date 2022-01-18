use std::path::PathBuf;

use eframe::{
	egui::{TextureId, Vec2},
	epi,
};
use image::{DynamicImage, GenericImageView};
use neos::AssetUrl;

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

/// This can block the whole thread for an API request, use with caution.
pub fn retrieve(url: &AssetUrl) -> Result<DynamicImage, String> {
	let path = get_path(url);

	if url.ext() == &Some("webp".to_owned()) {
		let bytes = match std::fs::read(path) {
			Ok(bytes) => bytes,
			Err(_) => fetch_asset(url)?,
		};

		let decoder = webp::Decoder::new(&bytes);

		let img = decoder
			.decode()
			.ok_or_else(|| "Failed to decode webp image".to_string())?;

		return Ok(img.to_image());
	}

	match image::io::Reader::open(&path) {
		Ok(file) => Ok(file.decode().map_err(|err| {
			"Failed to decode cached image: ".to_owned() + &err.to_string()
		})?),
		Err(_) => fetch_image(url),
	}
}

fn get_path(url: &AssetUrl) -> PathBuf {
	crate::TEMP_DIR.join(url.filename())
}

fn fetch_asset(url: &AssetUrl) -> Result<Vec<u8>, String> {
	let path = get_path(url);

	let res = minreq::get(url.to_string())
		.with_header("User-Agent", crate::USER_AGENT)
		.send()
		.map_err(|_| "Failed to send image request".to_owned())?;

	if res.status_code < 200 || res.status_code >= 300 {
		return Err("Image request status indicated failure".to_owned()
			+ &res.status_code.to_string());
	}

	let data = res.into_bytes();

	if let Err(err) = std::fs::write(path, &data) {
		println!("Failed to save asset {}: {}", url.filename(), err);
	}

	Ok(data)
}

fn fetch_image(url: &AssetUrl) -> Result<DynamicImage, String> {
	use std::io::Cursor;

	let data = fetch_asset(url)?;

	let img = image::io::Reader::new(Cursor::new(data))
		.with_guessed_format()
		.map_err(|err| {
			"Failed to parse fetched image data: ".to_owned() + &err.to_string()
		})?
		.decode()
		.map_err(|err| {
			"Failed to decode fetched image: ".to_owned() + &err.to_string()
		})?;

	Ok(img)
}
