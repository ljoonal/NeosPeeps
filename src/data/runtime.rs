use std::{
	cell::RefCell,
	collections::{HashMap, HashSet},
	rc::Rc,
	sync::Arc,
	time::SystemTime,
};

use ahash::RandomState;
use eframe::epi;
use neos::{
	api_client::{AnyNeos, NeosUnauthenticated},
	AssetUrl,
};

use super::{SessionWindow, TexturesMap, UserWindow};
use crate::{app::NeosPeepsApp, image::TextureDetails};

pub struct RuntimeOnly {
	pub password: String,
	pub totp: String,
	pub default_profile_picture: Option<Rc<TextureDetails>>,
	pub neos_api: Option<Arc<AnyNeos>>,
	pub friends: Vec<neos::NeosFriend>,
	/// Searched users.
	pub users: Vec<neos::NeosUser>,
	pub sessions: Vec<neos::NeosSession>,
	pub last_background_refresh: SystemTime,
	pub textures: TexturesMap,
	used_textures: RefCell<HashSet<String, RandomState>>,
	pub loading_textures: RefCell<HashSet<String, RandomState>>,
	pub user_window: RefCell<Option<UserWindow>>,
	pub session_window: RefCell<Option<SessionWindow>>,
}

impl Default for RuntimeOnly {
	fn default() -> Self {
		let api = NeosUnauthenticated::new(crate::USER_AGENT.to_owned());

		Self {
			totp: String::default(),
			password: String::default(),
			default_profile_picture: Option::default(),
			neos_api: Some(Arc::new(AnyNeos::Unauthenticated(api))),
			friends: Vec::default(),
			users: Vec::default(),
			sessions: Vec::default(),
			last_background_refresh: SystemTime::UNIX_EPOCH,
			textures: HashMap::default(),
			used_textures: RefCell::default(),
			loading_textures: RefCell::default(),
			user_window: RefCell::default(),
			session_window: RefCell::default(),
		}
	}
}

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
