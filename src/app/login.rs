//! The login page of the app

use super::NeosPeepsApp;
use eframe::{
	egui::{Button, ComboBox, SelectableLabel, TextEdit, Ui},
	epi,
};
use neos::{
	api_client::{
		AnyNeos, NeosRequestUserSession, NeosRequestUserSessionIdentifier,
		NeosUnauthenticated,
	},
	NeosUserSession,
};
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use std::thread;

impl NeosPeepsApp {
	/// Makes the current API try to use a session, or switch to unauthenticated
	/// on failure.
	pub fn try_use_session(
		&mut self,
		user_session: NeosUserSession,
		frame: epi::Frame,
	) {
		{
			let mut logging_in = self.logging_in.write().unwrap();
			if *logging_in {
				return;
			}
			*logging_in = true;
		}
		frame.request_repaint();

		let neos_api_arc = self.neos_api.clone();
		let logging_in = self.logging_in.clone();
		thread::spawn(move || {
			{
				let neos_api = NeosUnauthenticated::from(
					neos_api_arc.read().unwrap().clone(),
				)
				.upgrade(user_session);

				match neos_api.extend_session() {
					Ok(_) => {
						println!("Logged into Neos' API");
						*neos_api_arc.write().unwrap() = neos_api.into();
					}
					Err(err) => {
						*neos_api_arc.write().unwrap() =
							neos_api.downgrade().into();
						println!(
							"Error with Neos API user session extension: {}",
							err
						);
					}
				}
			}

			*logging_in.write().unwrap() = false;
			frame.request_repaint();
		});
	}

	pub fn login_new(
		&mut self,
		session_request: NeosRequestUserSession,
		frame: epi::Frame,
	) {
		{
			let mut logging_in = self.logging_in.write().unwrap();
			if *logging_in {
				return;
			}
			*logging_in = true;
		}
		frame.request_repaint();

		let neos_api_arc = self.neos_api.clone();
		let user_session = self.user_session.clone();
		let logging_in = self.logging_in.clone();
		thread::spawn(move || {
			let neos_api: NeosUnauthenticated =
				neos_api_arc.read().unwrap().clone().into();

			match neos_api.login(&session_request) {
				Ok(neos_user_session) => {
					println!("Logged in to Neos' API");
					*neos_api_arc.write().unwrap() =
						neos_api.upgrade(neos_user_session.clone()).into();
					*user_session.write().unwrap() = Some(neos_user_session);
				}
				Err(err) => {
					println!("Error with Neos API login request: {}", err);
				}
			}

			*logging_in.write().unwrap() = false;
			frame.request_repaint();
		});
	}

	pub fn logout(&mut self, frame: epi::Frame) {
		{
			let mut logging_in = self.logging_in.write().unwrap();
			if *logging_in {
				return;
			}
			*logging_in = true;
		}
		frame.request_repaint();

		let neos_api = self.neos_api.clone();
		let logging_in = self.logging_in.clone();
		thread::spawn(move || {
			let new_api = match neos_api.read().unwrap().clone() {
				AnyNeos::Authenticated(neos_api) => {
					neos_api.logout().ok();
					neos_api.downgrade()
				}
				AnyNeos::Unauthenticated(neos_api) => neos_api,
			};

			*neos_api.write().unwrap() = new_api.into();

			*logging_in.write().unwrap() = false;
			frame.request_repaint();
		});
	}

	fn identifier_picker(&mut self, ui: &mut Ui, is_loading: bool) {
		ComboBox::from_label("Login type")
			.selected_text(self.identifier.as_ref())
			.show_ui(ui, |ui| {
				if ui
					.add(SelectableLabel::new(
						matches!(
							self.identifier,
							NeosRequestUserSessionIdentifier::Username(_)
						),
						"Username",
					))
					.clicked()
				{
					self.identifier =
						NeosRequestUserSessionIdentifier::Username(
							self.identifier.inner().into(),
						);
				}

				if ui
					.add(SelectableLabel::new(
						matches!(
							self.identifier,
							NeosRequestUserSessionIdentifier::Email(_)
						),
						"Email",
					))
					.clicked()
				{
					self.identifier = NeosRequestUserSessionIdentifier::Email(
						self.identifier.inner().into(),
					);
				}

				if ui
					.add(SelectableLabel::new(
						matches!(
							self.identifier,
							NeosRequestUserSessionIdentifier::OwnerID(_)
						),
						"OwnerID",
					))
					.clicked()
				{
					self.identifier = NeosRequestUserSessionIdentifier::OwnerID(
						self.identifier.inner().into(),
					);
				}
			});

		let label = self.identifier.as_ref().to_string();

		ui.add(
			TextEdit::singleline(self.identifier.inner_mut())
				.hint_text(label)
				.interactive(!is_loading),
		);
	}

	pub fn login_page(&mut self, ui: &mut Ui, frame: epi::Frame) {
		let is_loading = *self.logging_in.read().unwrap();

		ui.heading("Log in");
		ui.label("Currently Neos' Oauth doesn't implement the required details for this application, thus logging in is the only way to actually use it.");
		if is_loading {
			ui.label("Logging in...");
		}

		ui.add_enabled_ui(!is_loading, |ui| {
			ui.group(|ui| {
				self.identifier_picker(ui, is_loading);

				ui.add(
					TextEdit::singleline(&mut self.password)
						.password(true)
						.hint_text("Password")
						.interactive(!is_loading),
				);

				let totp_resp = ui.add(
					TextEdit::singleline(&mut self.totp)
						.hint_text("2FA")
						.interactive(!is_loading)
						.desired_width(80_f32),
				);

				if totp_resp.changed() {
					self.totp = self
						.totp
						.chars()
						.filter(|v| v.is_numeric())
						.take(6)
						.collect();
				}

				let submit_button_resp = ui.add(Button::new("Log in"));

				if submit_button_resp.clicked()
					&& !self.identifier.inner().is_empty()
					&& !self.password.is_empty()
					&& !is_loading && (self.totp.is_empty()
					|| self.totp.chars().count() == 6)
				{
					let rand_string: String = thread_rng()
						.sample_iter(&Alphanumeric)
						.take(30)
						.map(char::from)
						.collect();
					let mut session_request =
						NeosRequestUserSession::with_identifier(
							self.identifier.clone(),
							std::mem::take(&mut self.password),
						)
						.remember_me(true)
						.machine_id(rand_string);

					if !self.totp.is_empty() {
						session_request = session_request
							.totp(std::mem::take(&mut self.totp));
					}

					self.login_new(session_request, frame);
				}
			});
		});
	}
}
