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

	pub fn credits_page(&mut self, ui: &mut Ui) {
		ui.heading("Credits & thanks");
		ui.label(concat!("Creating the app: ", env!("CARGO_PKG_AUTHORS")));
		ui.label("Posing for the screenshots: Soap");
		ui.label("Helping with URLs: kazu0617");
		ui.label("Initial help with neosdb urls: brodokk.");
		ui.label("Docs & code used as a reference: PolyLogiX.");
		ui.label("Pointing out smart screen issues with the .exe file: Ukilop.");
		ui.label("Good tutorials & smart comments: ProbablePrime.");
		ui.label("Good vibes: The Neos Modding discord");

		if ui.button("Back").clicked() {
			self.stored.page = Page::Peeps;
		}
	}

	pub fn license_page(&mut self, ui: &mut Ui) {
		for line in crate::LICENSE_TEXT.lines() {
			let trimmed_line = line.trim_start_matches('#');
			if trimmed_line == line {
				ui.label(trimmed_line.trim_start_matches(' '));
			} else {
				ui.heading(trimmed_line.trim_start_matches(' '));
			}
		}

		ui.separator();

		if ui.button("Back").clicked() {
			self.stored.page = Page::Peeps;
		}
	}
}
