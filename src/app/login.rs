//! The login page of the app

use super::NeosPeepsApp;
use crate::api::NeosApi;
use eframe::{
	egui::{Button, TextEdit, Ui},
	epi,
};
use std::thread;

impl NeosPeepsApp {
	pub fn login_page(&mut self, ui: &mut Ui, frame: epi::Frame) {
		let is_loading = *self.logging_in.read().unwrap();

		ui.heading("Log in");
		ui.label("Currently Neos' Oauth doesn't implement the required details for this application, thus logging in is the only way to actually use it.");
		if is_loading {
			ui.label("Logging in...");
		}
		ui.add_enabled(
			!is_loading,
			TextEdit::singleline(&mut self.username)
				.hint_text("Username")
				.interactive(!is_loading),
		);
		ui.add_enabled(
			!is_loading,
			TextEdit::singleline(&mut self.password)
				.password(true)
				.hint_text("Password")
				.interactive(!is_loading),
		);
		if ui.add_enabled(!is_loading, Button::new("Log in")).clicked()
			&& !self.username.is_empty()
			&& !self.password.is_empty()
			&& !is_loading
		{
			let (username, password) =
				(self.username.clone(), self.password.clone());
			self.password = String::new();
			println!("Going to try to login to the Neos api!");
			let neos_api_arc = self.neos_api.clone();
			let logging_in = self.logging_in.clone();
			thread::spawn(move || {
				*logging_in.write().unwrap() = true;
				frame.request_repaint();
				match NeosApi::try_login(&username, &password) {
					Ok(neos_api) => {
						println!("Logged in to Neos' API: {:?}", neos_api);
						*neos_api_arc.write().unwrap() = Some(neos_api);
					}
					Err(e) => println!("Error with Neos API: {}", e),
				}
				*logging_in.write().unwrap() = false;
				frame.request_repaint();
			});
		}
	}
}
