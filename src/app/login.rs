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

impl NeosPeepsApp {
	/// Makes the current API try to use a session, or switch to unauthenticated
	/// on failure.
	pub fn try_use_session(
		&mut self,
		user_session: NeosUserSession,
		frame: epi::Frame,
	) {
		{
			let mut loading = self.runtime.loading.write().unwrap();
			if loading.login_op() {
				return; // Only allow one login op at once
			}
			*loading = crate::data::LoadingState::LoggingIn;
		}
		frame.request_repaint();

		let neos_api_arc = self.runtime.neos_api.clone();
		let loading = self.runtime.loading.clone();
		rayon::spawn(move || {
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

			*loading.write().unwrap() = crate::data::LoadingState::None;
			frame.request_repaint();
		});
	}

	pub fn login_new(
		&mut self,
		session_request: NeosRequestUserSession,
		frame: epi::Frame,
	) {
		{
			let mut loading = self.runtime.loading.write().unwrap();
			if loading.login_op() {
				return;
			}
			*loading = crate::data::LoadingState::LoggingIn;
		}
		frame.request_repaint();

		let neos_api_arc = self.runtime.neos_api.clone();
		let user_session_arc = self.stored.user_session.clone();
		let loading = self.runtime.loading.clone();
		rayon::spawn(move || {
			let neos_api: NeosUnauthenticated =
				neos_api_arc.read().unwrap().clone().into();

			match neos_api.login(&session_request) {
				Ok(neos_user_session) => {
					println!("Logged in to Neos' API");
					*neos_api_arc.write().unwrap() =
						neos_api.upgrade(neos_user_session.clone()).into();
					*user_session_arc.write().unwrap() =
						Some(neos_user_session);
				}
				Err(err) => {
					println!("Error with Neos API login request: {}", err);
				}
			}

			*loading.write().unwrap() = crate::data::LoadingState::None;
			frame.request_repaint();
		});
	}

	pub fn logout(&mut self, frame: epi::Frame) {
		{
			let mut loading = self.runtime.loading.write().unwrap();
			if loading.login_op() {
				return;
			}
			*loading = crate::data::LoadingState::LoggingOut;
		}
		frame.request_repaint();

		let neos_api_arc = self.runtime.neos_api.clone();
		let loading = self.runtime.loading.clone();
		rayon::spawn(move || {
			let new_api = match neos_api_arc.read().unwrap().clone() {
				AnyNeos::Authenticated(neos_api) => {
					neos_api.logout().ok();
					neos_api.downgrade()
				}
				AnyNeos::Unauthenticated(neos_api) => neos_api,
			};

			*neos_api_arc.write().unwrap() = new_api.into();

			*loading.write().unwrap() = crate::data::LoadingState::None;
			frame.request_repaint();
		});
	}

	fn identifier_picker(&mut self, ui: &mut Ui, is_loading: bool) {
		ComboBox::from_label("Login type")
			.selected_text(self.stored.identifier.as_ref())
			.show_ui(ui, |ui| {
				if ui
					.add(SelectableLabel::new(
						matches!(
							self.stored.identifier,
							NeosRequestUserSessionIdentifier::Username(_)
						),
						"Username",
					))
					.clicked()
				{
					self.stored.identifier =
						NeosRequestUserSessionIdentifier::Username(
							self.stored.identifier.inner().into(),
						);
				}

				if ui
					.add(SelectableLabel::new(
						matches!(
							self.stored.identifier,
							NeosRequestUserSessionIdentifier::Email(_)
						),
						"Email",
					))
					.clicked()
				{
					self.stored.identifier =
						NeosRequestUserSessionIdentifier::Email(
							self.stored.identifier.inner().into(),
						);
				}

				if ui
					.add(SelectableLabel::new(
						matches!(
							self.stored.identifier,
							NeosRequestUserSessionIdentifier::OwnerID(_)
						),
						"OwnerID",
					))
					.clicked()
				{
					self.stored.identifier =
						NeosRequestUserSessionIdentifier::OwnerID(
							self.stored.identifier.inner().into(),
						);
				}
			});

		let label = self.stored.identifier.as_ref().to_string();

		ui.add(
			TextEdit::singleline(self.stored.identifier.inner_mut())
				.hint_text(label)
				.interactive(!is_loading),
		);
	}

	pub fn login_page(&mut self, ui: &mut Ui, frame: epi::Frame) {
		let is_loading = self.runtime.loading.read().unwrap().is_loading();

		ui.heading("Log in");
		ui.label("Currently Neos' Oauth doesn't implement the required details for this application, thus logging in is the only way to actually use it.");
		if is_loading {
			ui.label("Logging in...");
		}

		ui.add_enabled_ui(!is_loading, |ui| {
			ui.group(|ui| {
				self.identifier_picker(ui, is_loading);

				ui.add(
					TextEdit::singleline(&mut self.runtime.password)
						.password(true)
						.hint_text("Password")
						.interactive(!is_loading),
				);

				let totp_resp = ui.add(
					TextEdit::singleline(&mut self.runtime.totp)
						.hint_text("2FA")
						.interactive(!is_loading)
						.desired_width(80_f32),
				);

				if totp_resp.changed() {
					self.runtime.totp = self
						.runtime
						.totp
						.chars()
						.filter(|v| v.is_numeric())
						.take(6)
						.collect();
				}

				let submit_button_resp = ui.add(Button::new("Log in"));

				if submit_button_resp.clicked()
					&& !self.stored.identifier.inner().is_empty()
					&& !self.runtime.password.is_empty()
					&& !is_loading && (self.runtime.totp.is_empty()
					|| self.runtime.totp.chars().count() == 6)
				{
					let rand_string: String = thread_rng()
						.sample_iter(&Alphanumeric)
						.take(30)
						.map(char::from)
						.collect();
					let mut session_request =
						NeosRequestUserSession::with_identifier(
							self.stored.identifier.clone(),
							std::mem::take(&mut self.runtime.password),
						)
						.remember_me(true)
						.machine_id(rand_string);

					if !self.runtime.totp.is_empty() {
						session_request = session_request
							.totp(std::mem::take(&mut self.runtime.totp));
					}

					self.login_new(session_request, frame);
				}
			});
		});
	}
}
