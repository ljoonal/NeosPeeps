use std::rc::Rc;

use eframe::egui::Context;
use rayon::ThreadPool;

mod channels;
mod loading;

use channels::Channels;

use crate::app::NeosPeepsApp;

#[derive(Debug)]
pub struct Manager {
	pub channels: Channels,
	pub loading: loading::Tracker,
	data: ThreadPool,
	// Also logout operations
	login: ThreadPool,
}

impl Default for Manager {
	fn default() -> Self {
		Self {
			channels: Channels::default(),
			loading: loading::Tracker::default(),
			data: rayon::ThreadPoolBuilder::new()
				.panic_handler(move |m| {
					eprintln!("WARNING: Data thread panicked! {m:?}");
				})
				.build()
				.unwrap(),
			login: rayon::ThreadPoolBuilder::new()
				.num_threads(1)
				.panic_handler(move |m| {
					eprintln!("WARNING: Login thread panicked! {m:?}");
				})
				.build()
				.unwrap(),
		}
	}
}

impl Manager {
	/// Also for logout operations
	pub fn spawn_login_op<OP>(&self, op: OP)
	where
		OP: FnOnce() + Send + 'static,
	{
		if let Err(e) =
			std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
				self.login.spawn(op);
			})) {
			eprintln!("WARNING: Login thread panicked! {e:?}");
		}
	}

	/// Spawns a thread for fetching data from the API & so on.
	pub fn spawn_data_op<OP>(&self, op: OP)
	where
		OP: FnOnce() + Send + 'static,
	{
		if let Err(e) =
			std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
				self.data.spawn_fifo(op);
			})) {
			eprintln!("WARNING: Data thread panicked! {e:?}");
		}
	}
}

impl NeosPeepsApp {
	/// Tries to receive messages from other threads
	pub fn try_recv(&mut self, ctx: &Context) {
		let mut repaint = false;

		self.try_recv_auth(&mut repaint);
		self.try_recv_lists(&mut repaint);
		self.try_recv_window(&mut repaint);

		for (id, image) in self.threads.channels.try_recv_images() {
			self.runtime.loading_textures.get_mut().remove(&id);
			if let Some(image) = image {
				self.runtime.textures.insert(id, Rc::new(image));
			}
			repaint = true;
		}

		if let Some(latest_ver) = self.threads.channels.try_recv_updates() {
			self.runtime.available_update = Some(latest_ver);
		}

		if repaint {
			ctx.request_repaint();
		}
	}

	fn try_recv_auth(&mut self, repaint: &mut bool) {
		if let Some(user_session) = self.threads.channels.try_recv_user_session() {
			self.stored.user_session = user_session;
			*self.runtime.session_window.borrow_mut() = None;
			*self.runtime.user_window.borrow_mut() = None;
		}

		if let Some(client) = self.threads.channels.try_recv_auth() {
			self.runtime.neos_api = Some(client);
			*repaint = true;
		}
	}

	fn try_recv_lists(&mut self, repaint: &mut bool) {
		if let Some(res) = self.threads.channels.try_recv_friends() {
			self.threads.loading.friends.set(false);
			match res {
				Ok(friends) => {
					self.runtime.friends = friends;
					*repaint = true;
				}
				Err(e) => eprintln!("Failed to fetch friends! {e}"),
			}
		}

		if let Some(res) = self.threads.channels.try_recv_users() {
			self.threads.loading.users.set(false);
			match res {
				Ok(users) => {
					self.runtime.users = users;
					*repaint = true;
				}
				Err(e) => eprintln!("Failed to fetch users! {e}"),
			}
		}

		if let Some(res) = self.threads.channels.try_recv_sessions() {
			self.threads.loading.sessions.set(false);
			match res {
				Ok(sessions) => {
					self.runtime.sessions = sessions;
					*repaint = true;
				}
				Err(e) => eprintln!("Failed to fetch sessions! {e}"),
			}
		}

		if let Some(res) = self.threads.channels.try_recv_messages() {
			self.threads.loading.messages.set(false);
			match res {
				Ok(messages) => {
					for (user_id, fetched_messages) in messages {
						if let Some(stored_messages) =
							self.runtime.messages.get_mut(&user_id)
						{
							for message in fetched_messages.into_vec() {
								stored_messages.replace(message);
							}
						} else {
							self.runtime.messages.insert(user_id, fetched_messages);
						}
					}

					*repaint = true;
				}
				Err(e) => eprintln!("Failed to fetch messages! {e}"),
			}
		}
	}

	fn try_recv_window(&mut self, repaint: &mut bool) {
		if let Some(res) = self.threads.channels.try_recv_user() {
			self.threads.loading.user.set(false);
			match res {
				Ok(user) => {
					if let Some((user_id, w_user, _)) =
						&mut *self.runtime.user_window.borrow_mut()
					{
						if user.id == *user_id {
							*w_user = Some(user);
						}
					}
					*repaint = true;
				}
				Err(e) => eprintln!("Failed to fetch user! {e}"),
			}
		}

		if let Some(res) = self.threads.channels.try_recv_user_status() {
			self.threads.loading.user_status.set(false);
			match res {
				Ok((user_id, user_status)) => {
					if let Some((w_user_id, _, w_user_status)) =
						&mut *self.runtime.user_window.borrow_mut()
					{
						if user_id == *w_user_id {
							*w_user_status = Some(user_status);
						}
					}
					*repaint = true;
				}
				Err(e) => eprintln!("Failed to fetch user! {e}"),
			}
		}

		if let Some(res) = self.threads.channels.try_recv_session() {
			self.threads.loading.session.set(false);
			match res {
				Ok(session) => {
					if let Some((session_id, w_session)) =
						&mut *self.runtime.session_window.borrow_mut()
					{
						if session.id == *session_id {
							*w_session = Some(session);
						}
					}
					*repaint = true;
				}
				Err(e) => eprintln!("Failed to fetch user! {e}"),
			}
		}
	}
}
