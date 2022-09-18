use std::time::SystemTime;

use eframe::egui::{Align, Button, Context, Layout, Response, TextEdit, Ui};

use super::NeosPeepsApp;
use crate::data::Page;

impl NeosPeepsApp {
	fn add_page_button(&mut self, ui: &mut Ui, label: &str, page: Page) {
		if ui.add_enabled(self.stored.page != page, Button::new(label)).clicked() {
			self.stored.page = page;
			ui.close_menu();
		}
	}

	pub fn top_bar(
		&mut self, ui: &mut Ui, ctx: &Context, frame: &mut eframe::Frame,
	) {
		let is_authenticated =
			self.runtime.neos_api.as_ref().map_or(false, |a| a.is_authenticated());

		eframe::egui::menu::bar(ui, |ui| {
			// View menu
			ui.menu_button("View", |ui| {
				if is_authenticated {
					self.add_page_button(ui, "Peeps", Page::Peeps);
					ui.separator();
					self.add_page_button(ui, "Sessions", Page::Sessions);
					ui.separator();
				}
				self.add_page_button(ui, "Settings", Page::Settings);
				ui.separator();
				self.add_page_button(ui, "About", Page::About);
				ui.separator();
				self.add_page_button(ui, "Credits", Page::Credits);
				ui.separator();
				self.add_page_button(ui, "License", Page::License);
			});

			ui.separator();

			ui.menu_button("Network", |ui| {
				if is_authenticated {
					if ui.add(Button::new("Refresh lists")).clicked() {
						ui.close_menu();
						self.runtime.last_background_refresh = SystemTime::UNIX_EPOCH;
					}
					ui.separator();
					if ui.add(Button::new("Log out")).clicked() {
						ui.close_menu();
						self.logout(ctx);
					}
					ui.separator();
				}
				if ui.add(Button::new("Check for app updates")).clicked() {
					ui.close_menu();
					self.check_updates();
				}
			});

			ui.separator();

			ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
				if ui.button("Quit").clicked() {
					frame.close();
				}
				if self.threads.loading.any() {
					ui.label("...");
				}
			});
		});
	}

	pub fn search_bar(&mut self, ui: &mut Ui) -> Response {
		let mut resp = None;
		ui.horizontal(|ui| {
			resp = Some(
				ui.add(
					TextEdit::singleline(&mut self.stored.filter_search)
						.hint_text("Filter"),
				),
			);
			ui.checkbox(&mut self.stored.filter_friends_only, "Friends only?");
		});
		resp.unwrap()
	}
}
