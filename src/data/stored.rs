use std::time::{Duration, SystemTime};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct Stored {
	pub check_updates: bool,
	pub last_update_check_time: SystemTime,
	pub user_session: Option<neos::UserSession>,
	pub identifier: neos::LoginCredentialsIdentifier,
	pub refresh_frequency: Duration,
	pub page: Page,
	pub row_height: f32,
	pub col_min_width: f32,
	pub filter_friends_only: bool,
	pub filter_search: String,
	/// For formats, see https://docs.rs/chrono/latest/chrono/format/strftime/index.html
	pub datetime_format: String,
}

#[derive(Serialize, Deserialize, PartialEq, Eq)]
pub enum Page {
	About,
	Credits,
	Peeps,
	Sessions,
	Settings,
	License,
}

impl Default for Page {
	fn default() -> Self { Self::Peeps }
}

impl Default for Stored {
	fn default() -> Self {
		Self {
			last_update_check_time: SystemTime::now(),
			check_updates: false,
			user_session: None,
			identifier: neos::LoginCredentialsIdentifier::Username(String::default()),
			refresh_frequency: Duration::from_secs(120),
			page: Page::default(),
			row_height: 150_f32,
			col_min_width: 200f32,
			filter_friends_only: true,
			filter_search: String::new(),
			datetime_format: String::from("%X %x"),
		}
	}
}
