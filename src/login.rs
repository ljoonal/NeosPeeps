//! The login page of the app

use std::sync::Arc;

use eframe::epi;
use neos::{
	api_client::{AnyNeos, NeosRequestUserSession, NeosUnauthenticated},
	NeosUserSession,
};

use crate::app::NeosPeepsApp;

impl NeosPeepsApp {
	/// Makes the current API try to use a session, or switch to unauthenticated
	/// on failure.
	pub fn try_use_session(
		&mut self, user_session: NeosUserSession, frame: &epi::Frame,
	) {
		let neos_api_arc = match &self.runtime.neos_api {
			Some(api) => api.clone(),
			None => return,
		};
		self.runtime.neos_api = None;
		let auth_sender = self.threads.channels.auth_sender();
		let user_session_sender = self.threads.channels.user_session_sender();
		self.threads.spawn_login_op(move || {
			let neos_api = NeosUnauthenticated::from((*neos_api_arc).clone())
				.upgrade(user_session);

			match neos_api.extend_session() {
				Ok(_) => {
					match auth_sender.send(Arc::new(neos_api.into())) {
						Ok(_) => println!("Logged into Neos' API"),
						Err(err) => println!("Failed to send auth to main thread! {}", err),
					};
				}
				Err(err) => {
					match auth_sender.send(Arc::new(neos_api.downgrade().into())) {
						Ok(_) => {
							println!("Error with Neos API user session extension: {}", err);
						}
						Err(send_err) => println!(
							"Error with Neos API user session extension, and also to main thread failed! {} - {}",
							err, send_err
						),
					};

					if let Err(err) = user_session_sender.send(None) {
						println!("Failed to send user_session to main thread! {}", err);
					}
				}
			}
		});

		frame.request_repaint();
	}

	pub fn login_new(
		&mut self, session_request: NeosRequestUserSession, frame: &epi::Frame,
	) {
		let neos_api_arc = match &self.runtime.neos_api {
			Some(api) => api.clone(),
			None => return,
		};
		self.runtime.neos_api = None;
		let user_session_sender = self.threads.channels.user_session_sender();
		let auth_sender = self.threads.channels.auth_sender();
		self.threads.spawn_login_op(move || {
			let neos_api: NeosUnauthenticated = ((*neos_api_arc).clone()).into();

			match neos_api.login(&session_request) {
				Ok(neos_user_session) => {
					match auth_sender
						.send(Arc::new(neos_api.upgrade(neos_user_session.clone()).into()))
					{
						Ok(_) => println!("Logged into Neos' API"),
						Err(err) => println!("Failed to send auth to main thread! {}", err),
					};

					if let Err(err) = user_session_sender.send(Some(neos_user_session)) {
						println!("Failed to send user_session to main thread! {}", err);
					}
				}
				Err(err) => {
					println!("Error with Neos API login request: {}", err);
				}
			}
		});

		frame.request_repaint();
	}

	pub fn logout(&mut self, frame: &epi::Frame) {
		let neos_api_arc = match &mut self.runtime.neos_api {
			Some(api) => api.clone(),
			None => return,
		};
		self.runtime.neos_api = None;
		let user_session_sender = self.threads.channels.user_session_sender();
		let auth_sender = self.threads.channels.auth_sender();
		self.threads.spawn_login_op(move || {
			let new_api = match (*neos_api_arc).clone() {
				AnyNeos::Authenticated(neos_api) => {
					neos_api.logout().ok();
					neos_api.downgrade()
				}
				AnyNeos::Unauthenticated(neos_api) => neos_api,
			};

			if let Err(err) = auth_sender.send(Arc::new(new_api.into())) {
				println!("Failed to send auth to main thread! {}", err);
			}

			if let Err(err) = user_session_sender.send(None) {
				println!("Failed to send user_session to main thread! {}", err);
			}
		});

		frame.request_repaint();
	}
}
