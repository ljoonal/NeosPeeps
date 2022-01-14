#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![warn(clippy::cargo)]

const USER_AGENT: &str = concat!(
	env!("CARGO_PKG_NAME"),
	"/",
	env!("CARGO_PKG_VERSION"),
	" (",
	env!("CARGO_PKG_REPOSITORY"),
	")"
);

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
