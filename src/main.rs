#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![warn(clippy::cargo)]
// Not much can be done about it :/
#![allow(clippy::multiple_crate_versions)]

use std::path::PathBuf;

const LICENSE_TEXT: &str = include_str!("../LICENSE.md");

const USER_AGENT: &str = concat!(
	env!("CARGO_PKG_NAME"),
	"/",
	env!("CARGO_PKG_VERSION"),
	" (",
	env!("CARGO_PKG_REPOSITORY"),
	")"
);

lazy_static::lazy_static! {
	static ref TEMP_DIR: PathBuf = {
		let dir = std::env::temp_dir().join(env!("CARGO_PKG_NAME"));
		std::fs::create_dir_all(&dir).unwrap();
		dir.canonicalize().unwrap()
	};
}

mod app;
mod data;
mod image;
mod login;
mod messages;
mod sessions;
mod styling;
mod threading;
mod updating;
mod users;

fn main() {
	let app = app::NeosPeepsApp::default();
	let boxed_app = Box::new(app);
	let native_options = eframe::NativeOptions::default();
	eframe::run_native(boxed_app, native_options);
}
