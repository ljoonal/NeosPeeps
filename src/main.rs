#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![warn(clippy::cargo)]

use std::path::{PathBuf, MAIN_SEPARATOR};

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
mod styling;

fn main() {
	let app = app::NeosPeepsApp::default();
	let boxed_app = Box::new(app);
	let native_options = eframe::NativeOptions::default();
	eframe::run_native(boxed_app, native_options);
}
