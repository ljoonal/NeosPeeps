use neos::api_client::{AnyNeos, NeosUnauthenticated};
use std::{
	sync::{Arc, RwLock},
	time::{Duration, Instant},
};

use crate::image::TextureDetails;

pub type FriendAndPic = (neos::NeosFriend, Option<TextureDetails>);

pub struct RuntimeOnly {
	pub password: String,
	pub totp: String,
	pub loading: Arc<RwLock<LoadingState>>,
	pub default_profile_picture: Option<TextureDetails>,
	pub about_popup_showing: bool,
	pub neos_api: Arc<RwLock<AnyNeos>>,
	pub friends: Arc<RwLock<Vec<FriendAndPic>>>,
	pub last_friends_refresh: Arc<RwLock<Instant>>,
}

pub enum LoadingState {
	None,
	FetchingFriends,
	LoggingIn,
	LoggingOut,
}

impl LoadingState {
	pub const fn is_loading(&self) -> bool {
		!matches!(self, LoadingState::None)
	}

	pub const fn login_op(&self) -> bool {
		matches!(self, LoadingState::LoggingIn)
			|| matches!(self, LoadingState::LoggingOut)
	}
}

impl Default for RuntimeOnly {
	fn default() -> Self {
		let api = NeosUnauthenticated::new(crate::USER_AGENT.to_owned());

		Self {
			totp: String::default(),
			password: String::default(),
			loading: Arc::new(RwLock::new(LoadingState::None)),
			default_profile_picture: Option::default(),
			about_popup_showing: Default::default(),
			neos_api: Arc::new(RwLock::new(AnyNeos::Unauthenticated(api))),
			friends: Arc::default(),
			last_friends_refresh: Arc::new(RwLock::new(
				Instant::now() - Duration::from_secs(u64::MAX / 2),
			)),
		}
	}
}
