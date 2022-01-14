use neos::{
	api_client::{
		AnyNeos, NeosRequestUserSessionIdentifier, NeosUnauthenticated,
	},
	NeosUserSession,
};
use serde::{Deserialize, Serialize};
use std::{
	collections::HashMap,
	sync::{Arc, RwLock},
	time::{Duration, Instant},
};

use crate::image::TextureDetails;

#[derive(Serialize, Deserialize)]
pub struct Stored {
	pub user_session: Arc<RwLock<Option<NeosUserSession>>>,
	pub identifier: NeosRequestUserSessionIdentifier,
	pub refresh_frequency: Duration,
}

impl Default for Stored {
	fn default() -> Self {
		Self {
			user_session: Arc::default(),
			identifier: NeosRequestUserSessionIdentifier::Username(
				String::default(),
			),
			refresh_frequency: Duration::from_secs(120),
		}
	}
}

/// [`neos::AssetUrl`] ID's as keys.
pub type PicturesMap = HashMap<String, Option<TextureDetails>>;

pub struct RuntimeOnly {
	pub password: String,
	pub totp: String,
	pub loading: Arc<RwLock<LoadingState>>,
	pub default_profile_picture: Option<TextureDetails>,
	pub about_popup_showing: bool,
	pub neos_api: Arc<RwLock<AnyNeos>>,
	pub friends: Arc<RwLock<Vec<neos::NeosFriend>>>,
	pub friend_pics: Arc<RwLock<PicturesMap>>,
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
			friend_pics: Arc::default(),
			last_friends_refresh: Arc::new(RwLock::new(
				Instant::now() - Duration::from_secs(u64::MAX / 2),
			)),
		}
	}
}
