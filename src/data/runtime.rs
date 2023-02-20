use std::{
	cell::RefCell,
	collections::{HashMap, HashSet},
	rc::Rc,
	sync::Arc,
	time::SystemTime,
};

use ahash::RandomState;
use eframe::egui::{Context, TextureHandle, TextureOptions};
use neos::{
	api_client::{AnyNeos, NeosUnauthenticated},
	AssetUrl,
};
use time::{format_description::FormatItem, OffsetDateTime};

use super::{SessionWindow, TexturesMap, UserWindow, DEFAULT_TIME_FORMAT};
use crate::{
	app::NeosPeepsApp,
	messages::AllMessages,
	updating::GiteaReleasesResponse,
};

#[allow(clippy::module_name_repetitions)]
pub struct RuntimeOnly {
	pub password: String,
	pub totp: String,
	pub default_profile_picture: Option<Rc<TextureHandle>>,
	pub neos_api: Option<Arc<AnyNeos>>,
	pub friends: Vec<neos::Friend>,
	/// Searched users.
	pub users: Vec<neos::User>,
	pub sessions: Vec<neos::SessionInfo>,
	pub messages: AllMessages,
	pub last_background_refresh: SystemTime,
	pub textures: TexturesMap,
	used_textures: RefCell<HashSet<String, RandomState>>,
	pub loading_textures: RefCell<HashSet<String, RandomState>>,
	pub user_window: RefCell<Option<UserWindow>>,
	pub session_window: RefCell<Option<SessionWindow>>,
	pub open_chat: RefCell<Option<(neos::id::User, String, SystemTime)>>,
	pub available_update: Option<GiteaReleasesResponse>,
	pub time_format: Vec<FormatItem<'static>>,
}

impl RuntimeOnly {
	pub fn format_time(&self, time: &OffsetDateTime) -> String {
		time
			.format(&self.time_format)
			.unwrap_or_else(|_| "Time format err".to_string())
	}
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
			messages: AllMessages::default(),
			last_background_refresh: SystemTime::UNIX_EPOCH,
			textures: HashMap::default(),
			used_textures: RefCell::default(),
			loading_textures: RefCell::default(),
			user_window: RefCell::default(),
			session_window: RefCell::default(),
			open_chat: RefCell::default(),
			available_update: None,
			time_format: DEFAULT_TIME_FORMAT.to_owned(),
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
					let image =
						ctx.load_texture(asset_url.id(), image, TextureOptions::LINEAR);
					image_sender.send((asset_url.id().to_owned(), Some(image))).unwrap();
				}
				Err(err) => {
					image_sender.send((asset_url.id().to_owned(), None)).unwrap();
					eprintln!("Failed to fetch image! {err}");
				}
			}
		});
	}
}
