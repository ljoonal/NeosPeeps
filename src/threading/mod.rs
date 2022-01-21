use std::rc::Rc;

use eframe::epi;
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
					println!("WARNING: Data thread panicked! {:?}", m);
				})
				.build()
				.unwrap(),
			login: rayon::ThreadPoolBuilder::new()
				.num_threads(1)
				.panic_handler(move |m| {
					println!("WARNING: Login thread panicked! {:?}", m);
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
		self.login.spawn(op);
	}

	/// Spawns a thread for fetching data from the API & so on.
	pub fn spawn_data_op<OP>(&self, op: OP)
	where
		OP: FnOnce() + Send + 'static,
	{
		self.data.spawn_fifo(op);
	}
}

impl NeosPeepsApp {
	/// Tries to receive messages from other threads
	pub fn try_recv(&mut self, frame: &epi::Frame) {
		let mut repaint = false;

		if let Some(res) = self.threads.channels.try_recv_friends() {
			*self.threads.loading.friends.borrow_mut() = false;
			match res {
				Ok(friends) => {
					self.runtime.friends = friends;
					repaint = true;
				}
				Err(e) => println!("Failed to fetch friends! {}", e),
			}
		}

		if let Some(res) = self.threads.channels.try_recv_users() {
			*self.threads.loading.users.borrow_mut() = false;
			match res {
				Ok(users) => {
					self.runtime.users = users;
					repaint = true;
				}
				Err(e) => println!("Failed to fetch users! {}", e),
			}
		}

		if let Some(res) = self.threads.channels.try_recv_sessions() {
			*self.threads.loading.sessions.borrow_mut() = false;
			match res {
				Ok(sessions) => {
					self.runtime.sessions = sessions;
					repaint = true;
				}
				Err(e) => println!("Failed to fetch sessions! {}", e),
			}
		}

		if let Some(user_session) = self.threads.channels.try_recv_user_session() {
			self.stored.user_session = user_session;
			*self.runtime.session_window.borrow_mut() = None;
			*self.runtime.user_window.borrow_mut() = None;
		}

		if let Some(client) = self.threads.channels.try_recv_auth() {
			self.runtime.neos_api = Some(client);
			repaint = true;
		}

		for (id, image) in self.threads.channels.try_recv_images() {
			self.runtime.loading_textures.get_mut().remove(&id);
			if let Some(image) = image {
				self.runtime.textures.insert(id, Rc::new(image));
			}
			repaint = true;
		}

		if let Some(res) = self.threads.channels.try_recv_user() {
			*self.threads.loading.user.borrow_mut() = false;
			match res {
				Ok(user) => {
					if let Some((user_id, w_user, _)) =
						&mut *self.runtime.user_window.borrow_mut()
					{
						if user.id == *user_id {
							*w_user = Some(user);
						}
					}
					repaint = true;
				}
				Err(e) => println!("Failed to fetch user! {}", e),
			}
		}

		if let Some(res) = self.threads.channels.try_recv_user_status() {
			*self.threads.loading.user_status.borrow_mut() = false;
			match res {
				Ok((user_id, user_status)) => {
					if let Some((w_user_id, _, w_user_status)) =
						&mut *self.runtime.user_window.borrow_mut()
					{
						if user_id == *w_user_id {
							*w_user_status = Some(user_status);
						}
					}
					repaint = true;
				}
				Err(e) => println!("Failed to fetch user! {}", e),
			}
		}

		if let Some(res) = self.threads.channels.try_recv_session() {
			*self.threads.loading.session.borrow_mut() = false;
			match res {
				Ok(session) => {
					if let Some((session_id, w_session)) =
						&mut *self.runtime.session_window.borrow_mut()
					{
						if session.id == *session_id {
							*w_session = Some(session);
						}
					}
					repaint = true;
				}
				Err(e) => println!("Failed to fetch user! {}", e),
			}
		}

		if repaint {
			frame.request_repaint();
		}
	}
}
