use std::path::PathBuf;

use eframe::egui::ColorImage;
use image::DynamicImage;
use neos::AssetUrl;

#[allow(clippy::module_name_repetitions)]
// A lot taken from the egui example:
// https://github.com/emilk/egui/blob/master/eframe/examples/image.rs
pub fn from_dynamic_image(image: &DynamicImage) -> ColorImage {
	let image_buffer = image.to_rgba8();
	let size = [image.width() as usize, image.height() as usize];
	let pixels = image_buffer.into_vec();

	ColorImage::from_rgba_unmultiplied(size, &pixels)
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

	Ok(
		decoder
			.decode()
			.ok_or_else(|| format!("Failed to decode fetched webp image {:?}", url))?
			.to_image(),
	)
}

fn get_path(url: &AssetUrl) -> PathBuf { crate::TEMP_DIR.join(url.filename()) }

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
		eprintln!("Failed to save asset {:?} - {}", url, err);
	}

	Ok(data)
}
