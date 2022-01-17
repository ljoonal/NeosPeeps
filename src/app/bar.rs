use crate::data::Page;

use super::NeosPeepsApp;
use eframe::{
	egui::{Button, Ui},
	epi,
};

impl NeosPeepsApp {
	pub fn top_bar(&mut self, ui: &mut Ui, frame: &epi::Frame) {
		let is_authenticated =
			self.runtime.neos_api.read().unwrap().is_authenticated();
		let is_loading = self.runtime.loading.read().unwrap().is_loading();
		let is_logging_in = self.runtime.loading.read().unwrap().login_op();

		eframe::egui::menu::bar(ui, |ui| {
			// View menu
			ui.menu_button("View", |ui| {
				if ui
					.add_enabled(
						!matches!(self.stored.page, Page::About),
						Button::new("About"),
					)
					.clicked()
				{
					self.stored.page = Page::About;
					ui.close_menu();
				}

				if !is_logging_in && is_authenticated {
					if ui
						.add_enabled(
							!matches!(self.stored.page, Page::Friends),
							Button::new("Friends"),
						)
						.clicked()
					{
						self.stored.page = Page::Friends;
						ui.close_menu();
					}

					#[cfg(debug_assertions)]
					if ui
						.add_enabled(
							!matches!(self.stored.page, Page::Sessions),
							Button::new("Sessions"),
						)
						.clicked()
					{
						self.stored.page = Page::Sessions;
						ui.close_menu();
					}
				}

				if ui
					.add_enabled(
						!matches!(self.stored.page, Page::Settings),
						Button::new("Settings"),
					)
					.clicked()
				{
					self.stored.page = Page::Settings;
					ui.close_menu();
				}
			});

			ui.separator();

			// Account menu.
			if !is_logging_in && is_authenticated {
				ui.menu_button("Account", |ui| {
					if ui
						.add_enabled(!is_loading, Button::new("Refresh"))
						.clicked()
					{
						ui.close_menu();
						self.refresh_friends(frame.clone());
					}
					ui.separator();
					if ui.add(Button::new("Log out")).clicked() {
						ui.close_menu();
						self.logout(frame.clone());
					}
				});
				ui.separator();
			}

			if ui.button("Quit").clicked() {
				frame.quit();
			}
		});
	}
}
