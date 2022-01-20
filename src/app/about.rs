//! The about page of the app

use eframe::egui::{warn_if_debug_build, Ui};

use super::NeosPeepsApp;
use crate::data::Page;

impl NeosPeepsApp {
	pub fn about_page(&mut self, ui: &mut Ui) {
		ui.heading(concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION")));
		warn_if_debug_build(ui);
		ui.label(concat!(
			env!("CARGO_PKG_NAME"),
			" is a tool that lists your NeosVR friends."
		));
		ui.label("It's purpose is to provide a more lightweight & quicker experience than launching the game.");
		ui.label(
			"Or alternatively for desktop users to have it on a second monitor.",
		);
		ui.spacing();
		ui.label(concat!(
			env!("CARGO_PKG_NAME"),
			" is an unofficial tool, and is not affiliated with the developers of NeosVR."
		));
		ui.hyperlink_to(
			"This application is licensed under AGPL. Get the source code here.",
			env!("CARGO_PKG_REPOSITORY"),
		);
		ui.spacing();

		if ui.button("Back").clicked() {
			self.stored.page = Page::Peeps;
		}
	}
}
