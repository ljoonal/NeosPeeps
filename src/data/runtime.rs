use std::{
	cell::RefCell,
	collections::{HashMap, HashSet},
	rc::Rc,
	sync::Arc,
	time::SystemTime,
};

use ahash::RandomState;
use neos::api_client::{AnyNeos, NeosUnauthenticated};

use super::{SessionWindow, TexturesMap, UserWindow};
use crate::image::TextureDetails;

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
