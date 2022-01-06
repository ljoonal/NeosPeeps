#![warn(clippy::all)]

const USER_AGENT: &str = concat!(
	env!("CARGO_PKG_NAME"),
	"/",
	env!("CARGO_PKG_VERSION"),
	" (",
	env!("CARGO_PKG_REPOSITORY"),
	")"
);

mod api;
mod app;
mod image;
mod styling;

fn main() {
	let app = app::NeosPeepsApp::default();
	let native_options = eframe::NativeOptions::default();
	eframe::run_native(Box::new(app), native_options);
}
