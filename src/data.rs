use crate::image::TextureDetails;
use ahash::RandomState;
use eframe::epi;
use neos::{
	api_client::{
		AnyNeos, NeosRequestUserSessionIdentifier, NeosUnauthenticated,
	},
	AssetUrl, NeosUserSession,
};
use serde::{Deserialize, Serialize};
use std::{
	collections::{HashMap, HashSet},
	sync::{Arc, RwLock},
	time::{Duration, SystemTime},
};

#[derive(Serialize, Deserialize)]
pub struct Stored {
	pub user_session: Arc<RwLock<Option<NeosUserSession>>>,
	pub identifier: NeosRequestUserSessionIdentifier,
	pub refresh_frequency: Duration,
	pub page: Page,
	pub row_height: f32,
}

#[derive(Serialize, Deserialize)]
pub enum Page {
	About,
	Friends,
	Sessions,
	Settings,
}

impl Default for Page {
	fn default() -> Self {
		Self::Friends
	}
}

impl Default for Stored {
	fn default() -> Self {
		Self {
			user_session: Arc::default(),
			identifier: NeosRequestUserSessionIdentifier::Username(
				String::default(),
			),
			refresh_frequency: Duration::from_secs(120),
			page: Page::default(),
			row_height: 150_f32,
		}
	}
}

/// [`neos::AssetUrl`] ID's as keys.
pub type TexturesMap =
	HashMap<String, Option<Arc<TextureDetails>>, RandomState>;

pub struct RuntimeOnly {
	pub password: String,
	pub totp: String,
	pub loading: Arc<RwLock<LoadingState>>,
	pub default_profile_picture: Option<Arc<TextureDetails>>,
	pub neos_api: Arc<RwLock<AnyNeos>>,
	pub friends: Arc<RwLock<Vec<neos::NeosFriend>>>,
	pub sessions: Arc<RwLock<Vec<neos::NeosSession>>>,
	pub last_background_refresh: Arc<RwLock<SystemTime>>,
	textures: Arc<RwLock<TexturesMap>>,
	used_textures: RwLock<HashSet<String, RandomState>>,
}

impl RuntimeOnly {
	pub fn cull_textures(&self) {
		let used_textures =
			std::mem::take(&mut *self.used_textures.write().unwrap());
		// Not having the same write locks at the same time is not ideal but
		// better for performance most likely.
		self.textures
			.write()
			.unwrap()
			.retain(|id, _| used_textures.contains(id));
	}
	pub fn load_texture(
		&self,
		asset_url: &AssetUrl,
		frame: &epi::Frame,
	) -> Option<Arc<TextureDetails>> {
		self.used_textures.write().unwrap().insert(asset_url.id().to_owned());
		if let Some(texture) = self.textures.read().ok()?.get(asset_url.id()) {
			if let Some(texture) = texture {
				return Some(texture.clone());
			}
		} else {
			self.start_retrieving_image(asset_url.clone(), frame.clone());
		}

		None
	}

	/// Starts a thread to start retrieving the image.
	fn start_retrieving_image(&self, asset_url: AssetUrl, frame: epi::Frame) {
		let textures = self.textures.clone();
		rayon::spawn_fifo(move || {
			textures.write().unwrap().insert(asset_url.id().to_owned(), None);
			match crate::image::retrieve(&asset_url) {
				Ok(image) => {
					let (size, image) = crate::image::to_epi_format(&image);
					let val = Some(Arc::new(TextureDetails::new(
						frame.clone(),
						size,
						image,
					)));
					textures
						.write()
						.unwrap()
						.insert(asset_url.id().to_owned(), val);
					frame.request_repaint();
				}
				Err(err) => {
					textures.write().unwrap().remove(asset_url.id());
					println!("Failed to fetch image: {}", err);
				}
			}
		});
	}
}

#[derive(Debug)]
pub enum LoginOperationState {
	None,
	LoggingIn,
	LoggingOut,
}

impl Default for LoginOperationState {
	fn default() -> Self {
		Self::None
	}
}

#[derive(Default, Debug)]
pub struct LoadingState {
	pub fetching_friends: bool,
	pub fetching_sessions: bool,
	pub login: LoginOperationState,
}

impl LoadingState {
	pub const fn is_loading(&self) -> bool {
		self.fetching_friends || self.fetching_sessions || self.login_op()
	}

	pub const fn login_op(&self) -> bool {
		!matches!(self.login, LoginOperationState::None)
	}
}

impl Default for RuntimeOnly {
	fn default() -> Self {
		let api = NeosUnauthenticated::new(crate::USER_AGENT.to_owned());

		Self {
			totp: String::default(),
			password: String::default(),
			loading: Arc::default(),
			default_profile_picture: Option::default(),
			neos_api: Arc::new(RwLock::new(AnyNeos::Unauthenticated(api))),
			friends: Arc::default(),
			sessions: Arc::default(),
			last_background_refresh: Arc::new(RwLock::new(
				SystemTime::UNIX_EPOCH,
			)),
			textures: Arc::default(),
			used_textures: RwLock::default(),
		}
	}
}
