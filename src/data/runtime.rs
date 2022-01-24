use std::{
	cell::RefCell,
	collections::{HashMap, HashSet},
	rc::Rc,
	sync::Arc,
	time::SystemTime,
};

use ahash::RandomState;
use eframe::egui::{Context, TextureHandle};
use neos::{
	api_client::{AnyNeos, NeosUnauthenticated},
	AssetUrl,
};

use super::{SessionWindow, TexturesMap, UserWindow};
use crate::app::NeosPeepsApp;

#[allow(clippy::module_name_repetitions)]
pub struct RuntimeOnly {
	pub password: String,
	pub totp: String,
	pub default_profile_picture: Option<Rc<TextureHandle>>,
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
		&self, asset_url: &AssetUrl, ctx: &Context,
	) -> Option<Rc<TextureHandle>> {
		self.runtime.used_textures.borrow_mut().insert(asset_url.id().to_owned());
		if let Some(texture) = self.runtime.textures.get(asset_url.id()) {
			return Some(texture.clone());
		}
		self.start_retrieving_image(asset_url.clone(), ctx.clone());

		None
	}

	/// Starts a thread to start retrieving the image if wasn't already.
	fn start_retrieving_image(&self, asset_url: AssetUrl, ctx: Context) {
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
					let image = crate::image::from_dynamic_image(&image);
					let image = ctx.load_texture(asset_url.id(), image);
					image_sender.send((asset_url.id().to_owned(), Some(image))).unwrap();
				}
				Err(err) => {
					image_sender.send((asset_url.id().to_owned(), None)).unwrap();
					println!("Failed to fetch image! {}", err);
				}
			}
		});
	}
}
