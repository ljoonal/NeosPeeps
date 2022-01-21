use std::{collections::HashMap, rc::Rc};

use ahash::RandomState;
use eframe::epi;
use neos::{AssetUrl, NeosSession, NeosUser, NeosUserStatus};

use crate::{app::NeosPeepsApp, image::TextureDetails};

mod runtime;
mod stored;

/// [`neos::AssetUrl`] ID's as keys.
pub type TexturesMap = HashMap<String, Rc<TextureDetails>, RandomState>;

pub type UserWindow =
	(neos::id::User, Option<NeosUser>, Option<NeosUserStatus>);
pub type SessionWindow = (neos::id::Session, Option<NeosSession>);

impl NeosPeepsApp {
	pub fn cull_textures(&mut self) {
		let used_textures =
			std::mem::take(&mut self.runtime.used_textures).into_inner();
		self.runtime.textures.retain(|id, _| used_textures.contains(id));
	}

	pub fn load_texture(
		&self, asset_url: &AssetUrl, frame: &epi::Frame,
	) -> Option<Rc<TextureDetails>> {
		self.runtime.used_textures.borrow_mut().insert(asset_url.id().to_owned());
		if let Some(texture) = self.runtime.textures.get(asset_url.id()) {
			return Some(texture.clone());
		}
		self.start_retrieving_image(asset_url.clone(), frame.clone());

		None
	}

	/// Starts a thread to start retrieving the image if wasn't already.
	fn start_retrieving_image(&self, asset_url: AssetUrl, frame: epi::Frame) {
		if !self
			.runtime
			.loading_textures
			.borrow_mut()
			.insert(asset_url.id().to_string())
		{
			return;
		}
		let image_sender = self.threads.channels.image_sender();
		self.threads.spawn_data_op(move || {
			match crate::image::retrieve(&asset_url) {
				Ok(image) => {
					let (size, image) = crate::image::to_epi_format(&image);
					let image = Some(TextureDetails::new(frame, size, image));
					if let Err(err) =
						image_sender.send((asset_url.id().to_owned(), image))
					{
						println!("Couldn't send image to main thread! {}", err);
					}
				}
				Err(err) => {
					match image_sender.send((asset_url.id().to_owned(), None)) {
						Ok(_) => println!("Failed to fetch image! {}", err),
						Err(thread_err) => println!(
							"Failed to fetch image & to send to main thread: {} - {}",
							err, thread_err
						),
					};
				}
			}
		});
	}
}
