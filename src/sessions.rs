use eframe::epi;
use neos::api_client::{AnyNeos, Neos};

use crate::app::NeosPeepsApp;

impl NeosPeepsApp {
	/// Refreshes sessions in a background thread
	pub fn refresh_sessions(&mut self, frame: &epi::Frame) {
		use rayon::prelude::*;

		let neos_api_arc = match &self.runtime.neos_api {
			Some(api) => api.clone(),
			None => return,
		};

		self.threads.loading.sessions.set(true);
		let sessions_sender = self.threads.channels.sessions_sender();
		self.threads.spawn_data_op(move || {
			if let AnyNeos::Authenticated(neos_api) = &*neos_api_arc {
				match neos_api.get_sessions() {
					Ok(mut sessions) => {
						sessions.par_sort_by(|s1, s2| {
							s1.active_users.cmp(&s2.active_users).reverse()
						});
						sessions_sender.send(Ok(sessions)).unwrap();
					}
					Err(e) => sessions_sender.send(Err(e.to_string())).unwrap(),
				}
			}
		});

		frame.request_repaint();
	}

	/// Gets the session status for the session window
	pub fn get_session(&self, frame: &epi::Frame, id: &neos::id::Session) {
		let neos_api = match &self.runtime.neos_api {
			Some(api) => api.clone(),
			None => return,
		};

		if let Some((w_id, _)) = &*self.runtime.session_window.borrow() {
			if w_id != id {
				return;
			}
		} else {
			*self.runtime.session_window.borrow_mut() = Some((id.clone(), None));
		}

		self.threads.loading.session.set(true);
		let id = id.clone();
		let session_sender = self.threads.channels.session_sender();
		self.threads.spawn_data_op(move || {
			let res = neos_api.get_session(id);
			session_sender.send(res.map_err(|e| e.to_string())).unwrap();
		});

		frame.request_repaint();
	}
}

pub fn find_focused_session<'a>(
	id: &neos::id::User, user_status: &'a neos::UserStatus,
) -> Option<&'a neos::SessionInfo> {
	use rayon::prelude::*;

	user_status.active_sessions.par_iter().find_any(|session| {
		session
			.users
			.par_iter()
			.find_any(|user| match &user.id {
				Some(user_id) => user_id == id && user.is_present,
				None => false,
			})
			.is_some()
	})
}
