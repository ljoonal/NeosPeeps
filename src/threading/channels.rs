use std::{rc::Rc, sync::Arc};

use crossbeam::channel::{bounded, unbounded, Receiver, Sender, TryIter};
use eframe::epi;
use neos::{
	api_client::AnyNeos,
	NeosFriend,
	NeosSession,
	NeosUser,
	NeosUserSession,
	NeosUserStatus,
};

use crate::{app::NeosPeepsApp, image::TextureDetails};

type ImageMsg = (String, Option<TextureDetails>);
type UserStatusMsg = (neos::id::User, NeosUserStatus);

// Sender & Receiver than can have errors.
type Res<T> = Result<T, String>;
type ResSender<T> = Sender<Res<T>>;
type ResReceiver<T> = Receiver<Res<T>>;

#[derive(Debug)]
pub struct Channels {
	/// Friends bg refresh
	friends: (ResSender<Vec<NeosFriend>>, ResReceiver<Vec<NeosFriend>>),
	/// Users search
	users: (ResSender<Vec<NeosUser>>, ResReceiver<Vec<NeosUser>>),
	/// Sessions bg refresh
	sessions: (ResSender<Vec<NeosSession>>, ResReceiver<Vec<NeosSession>>),
	/// Login/Logout
	auth: (Sender<Arc<AnyNeos>>, Receiver<Arc<AnyNeos>>),
	/// New login was successful
	user_session:
		(Sender<Option<NeosUserSession>>, Receiver<Option<NeosUserSession>>),
	/// Image assets being loaded
	image: (Sender<ImageMsg>, Receiver<ImageMsg>),
	/// Lookups for the user window
	user: (ResSender<NeosUser>, ResReceiver<NeosUser>),
	/// Lookups for the user window
	user_status: (ResSender<UserStatusMsg>, ResReceiver<UserStatusMsg>),
	/// Lookups for the session window
	session: (ResSender<NeosSession>, ResReceiver<NeosSession>),
}

impl Default for Channels {
	fn default() -> Self {
		Self {
			friends: bounded(1),
			users: unbounded(),
			auth: bounded(1),
			sessions: bounded(1),
			user_session: bounded(1),
			image: unbounded(),
			user: bounded(1),
			user_status: bounded(1),
			session: bounded(1),
		}
	}
}

// Allow trying to receive or getting a sender but nothing more.
impl Channels {
	pub fn friends_sender(&self) -> ResSender<Vec<NeosFriend>> {
		self.friends.0.clone()
	}

	pub fn users_sender(&self) -> ResSender<Vec<NeosUser>> {
		self.users.0.clone()
	}

	pub fn sessions_sender(&self) -> ResSender<Vec<NeosSession>> {
		self.sessions.0.clone()
	}

	pub fn auth_sender(&self) -> Sender<Arc<AnyNeos>> { self.auth.0.clone() }

	pub fn user_session_sender(&self) -> Sender<Option<NeosUserSession>> {
		self.user_session.0.clone()
	}

	pub fn image_sender(&self) -> Sender<ImageMsg> { self.image.0.clone() }

	pub fn user_sender(&self) -> ResSender<NeosUser> { self.user.0.clone() }

	pub fn user_status_sender(&self) -> ResSender<UserStatusMsg> {
		self.user_status.0.clone()
	}

	pub fn session_sender(&self) -> ResSender<NeosSession> {
		self.session.0.clone()
	}

	pub fn try_recv_friends(&self) -> Option<Res<Vec<NeosFriend>>> {
		self.friends.1.try_recv().ok()
	}

	pub fn try_recv_users(&self) -> Option<Res<Vec<NeosUser>>> {
		self.users.1.try_recv().ok()
	}

	pub fn try_recv_sessions(&self) -> Option<Res<Vec<NeosSession>>> {
		self.sessions.1.try_recv().ok()
	}

	pub fn try_recv_auth(&self) -> Option<Arc<AnyNeos>> {
		self.auth.1.try_recv().ok()
	}

	#[allow(clippy::option_option)]
	pub fn try_recv_user_session(&self) -> Option<Option<NeosUserSession>> {
		self.user_session.1.try_recv().ok()
	}

	pub fn try_recv_images(&self) -> TryIter<ImageMsg> { self.image.1.try_iter() }

	pub fn try_recv_user(&self) -> Option<Res<NeosUser>> {
		self.user.1.try_recv().ok()
	}

	pub fn try_recv_user_status(&self) -> Option<Res<UserStatusMsg>> {
		self.user_status.1.try_recv().ok()
	}

	pub fn try_recv_session(&self) -> Option<Res<NeosSession>> {
		self.session.1.try_recv().ok()
	}
}

impl NeosPeepsApp {
	/// Tries to receive messages from other threads
	pub fn try_recv(&mut self, frame: &epi::Frame) {
		let mut repaint = false;

		if let Some(res) = self.threads.channels.try_recv_friends() {
			match res {
				Ok(friends) => {
					self.runtime.friends = friends;
					repaint = true;
				}
				Err(e) => println!("Failed to fetch friends! {}", e),
			}
		}

		if let Some(res) = self.threads.channels.try_recv_users() {
			match res {
				Ok(users) => {
					self.runtime.users = users;
					repaint = true;
				}
				Err(e) => println!("Failed to fetch users! {}", e),
			}
		}

		if let Some(res) = self.threads.channels.try_recv_sessions() {
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
