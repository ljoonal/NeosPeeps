use std::time::{Duration, SystemTime};

use serde::{Deserialize, Serialize};
use time::format_description::FormatItem;
use time::macros::format_description;

pub const DEFAULT_TIME_FORMAT_STR: &str =
	"[hour]:[minute]:[second] [day].[month].[year]";
pub const DEFAULT_TIME_FORMAT: &[FormatItem<'static>] =
	format_description!("[hour]:[minute]:[second] [day].[month].[year]");

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
	/// For formats, see https://time-rs.github.io/book/api/format-description.html
	pub time_format: String,
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
			time_format: DEFAULT_TIME_FORMAT_STR.to_owned(),
		}
	}
}
