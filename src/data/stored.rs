use std::time::Duration;

use neos::{api_client::NeosRequestUserSessionIdentifier, NeosUserSession};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct Stored {
	pub user_session: Option<NeosUserSession>,
	pub identifier: NeosRequestUserSessionIdentifier,
	pub refresh_frequency: Duration,
	pub page: Page,
	pub row_height: f32,
	pub col_min_width: f32,
	pub filter_friends_only: bool,
	pub filter_search: String,
	/// For formats, see https://docs.rs/chrono/latest/chrono/format/strftime/index.html
	pub datetime_format: String,
}

#[derive(Serialize, Deserialize)]
pub enum Page {
	About,
	Peeps,
	Sessions,
	Settings,
}

impl Default for Page {
	fn default() -> Self { Self::Peeps }
}

impl Default for Stored {
	fn default() -> Self {
		Self {
			user_session: None,
			identifier: NeosRequestUserSessionIdentifier::Username(String::default()),
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
