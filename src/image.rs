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
	use image::ImageFormat;

	let path = get_path(url);

	let bytes = match std::fs::read(path) {
		Ok(bytes) => bytes,
		Err(_) => fetch_asset(url)?,
	};

	let format = image::guess_format(&bytes).map_err(|err| {
		format!("Failed to guess format of fetched image {:?} - {}", url, err)
	})?;

	if format != ImageFormat::WebP {
		return image::load_from_memory_with_format(&bytes, format).map_err(
			|err| format!("Failed to decode fetched image {:?} - {}", url, err),
		);
	}

	let decoder = webp::Decoder::new(&bytes);

	Ok(decoder
		.decode()
		.ok_or_else(|| {
			format!("Failed to decode fetched webp image {:?}", url)
		})?
		.to_image())
}

fn get_path(url: &AssetUrl) -> PathBuf {
	crate::TEMP_DIR.join(url.filename())
}

fn fetch_asset(url: &AssetUrl) -> Result<Vec<u8>, String> {
	let path = get_path(url);

	let res = minreq::get(url.to_string())
		.with_header("User-Agent", crate::USER_AGENT)
		.send()
		.map_err(|err| {
			format!("Failed to send image request {:?} - {}", url, err)
		})?;

	if res.status_code < 200 || res.status_code >= 300 {
		return Err(format!(
			"Image request status indicated failure {:?} - {}",
			url, res.status_code,
		));
	}

	let data = res.into_bytes();

	if let Err(err) = std::fs::write(path, &data) {
		println!("Failed to save asset {:?} - {}", url, err);
	}

	Ok(data)
}
