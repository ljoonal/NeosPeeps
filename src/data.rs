use crate::{app::NeosPeepsApp, image::TextureDetails};
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
	cell::RefCell,
	collections::{HashMap, HashSet},
	rc::Rc,
	sync::Arc,
	time::{Duration, SystemTime},
};

#[derive(Serialize, Deserialize)]
pub struct Stored {
	pub user_session: Option<NeosUserSession>,
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
			user_session: None,
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
pub type TexturesMap = HashMap<String, Rc<TextureDetails>, RandomState>;

pub struct RuntimeOnly {
	pub password: String,
	pub totp: String,
	pub loading: LoadingState,
	pub default_profile_picture: Option<Rc<TextureDetails>>,
	pub neos_api: Arc<AnyNeos>,
	pub friends: Vec<neos::NeosFriend>,
	pub sessions: Vec<neos::NeosSession>,
	pub last_background_refresh: SystemTime,
	pub textures: TexturesMap,
	used_textures: RefCell<HashSet<String, RandomState>>,
	pub loading_textures: RefCell<HashSet<String, RandomState>>,
}

impl NeosPeepsApp {
	pub fn cull_textures(&mut self) {
		let used_textures =
			std::mem::take(&mut self.runtime.used_textures).into_inner();
		self.runtime.textures.retain(|id, _| used_textures.contains(id));
	}
	pub fn load_texture(
		&self,
		asset_url: &AssetUrl,
		frame: &epi::Frame,
	) -> Option<Rc<TextureDetails>> {
		self.runtime
			.used_textures
			.borrow_mut()
			.insert(asset_url.id().to_owned());
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
		let image_sender = self.channels.image_sender();
		rayon::spawn_fifo(move || match crate::image::retrieve(&asset_url) {
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
				match image_sender.send((
					asset_url.id().to_owned(),
					None,
				)) {
					Ok(_) => println!("Failed to fetch image! {}", err),
					Err(thread_err) =>  println!("Failed to fetch image & to send to main thread: {} - {}", err, thread_err)
				};
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
			loading: LoadingState::default(),
			default_profile_picture: Option::default(),
			neos_api: Arc::new(AnyNeos::Unauthenticated(api)),
			friends: Vec::default(),
			sessions: Vec::default(),
			last_background_refresh: SystemTime::UNIX_EPOCH,
			textures: HashMap::default(),
			used_textures: RefCell::default(),
			loading_textures: RefCell::default(),
		}
	}
}
