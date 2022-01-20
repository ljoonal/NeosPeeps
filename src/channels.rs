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

use crate::{
	app::NeosPeepsApp,
	data::LoginOperationState,
	image::TextureDetails,
};

type ImageMsg = (String, Option<TextureDetails>);
type UserStatusMsg = (neos::id::User, NeosUserStatus);

pub struct Channels {
	/// Friends bg refresh
	friends: (Sender<Vec<NeosFriend>>, Receiver<Vec<NeosFriend>>),
	/// Users search
	users: (Sender<Vec<NeosUser>>, Receiver<Vec<NeosUser>>),
	/// Sessions bg refresh
	sessions: (Sender<Vec<NeosSession>>, Receiver<Vec<NeosSession>>),
	/// Login/Logout
	auth: (Sender<Arc<AnyNeos>>, Receiver<Arc<AnyNeos>>),
	/// New login was successful
	user_session:
		(Sender<Option<NeosUserSession>>, Receiver<Option<NeosUserSession>>),
	/// Image assets being loaded
	image: (Sender<ImageMsg>, Receiver<ImageMsg>),
	/// Lookups for the user window
	user: (Sender<NeosUser>, Receiver<NeosUser>),
	/// Lookups for the user window
	user_status: (Sender<UserStatusMsg>, Receiver<UserStatusMsg>),
	/// Lookups for the session window
	session: (Sender<NeosSession>, Receiver<NeosSession>),
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
	pub fn friends_sender(&self) -> Sender<Vec<NeosFriend>> {
		self.friends.0.clone()
	}
	pub fn users_sender(&self) -> Sender<Vec<NeosUser>> {
		self.users.0.clone()
	}
	pub fn sessions_sender(&self) -> Sender<Vec<NeosSession>> {
		self.sessions.0.clone()
	}
	pub fn auth_sender(&self) -> Sender<Arc<AnyNeos>> {
		self.auth.0.clone()
	}
	pub fn user_session_sender(&self) -> Sender<Option<NeosUserSession>> {
		self.user_session.0.clone()
	}
	pub fn image_sender(&self) -> Sender<ImageMsg> {
		self.image.0.clone()
	}
	pub fn user_sender(&self) -> Sender<NeosUser> {
		self.user.0.clone()
	}
	pub fn user_status_sender(&self) -> Sender<UserStatusMsg> {
		self.user_status.0.clone()
	}
	pub fn session_sender(&self) -> Sender<NeosSession> {
		self.session.0.clone()
	}

	pub fn try_recv_friends(&self) -> Option<Vec<NeosFriend>> {
		self.friends.1.try_recv().ok()
	}

	pub fn try_recv_users(&self) -> Option<Vec<NeosUser>> {
		self.users.1.try_recv().ok()
	}

	pub fn try_recv_sessions(&self) -> Option<Vec<NeosSession>> {
		self.sessions.1.try_recv().ok()
	}

	pub fn try_recv_auth(&self) -> Option<Arc<AnyNeos>> {
		self.auth.1.try_recv().ok()
	}

	#[allow(clippy::option_option)]
	pub fn try_recv_user_session(&self) -> Option<Option<NeosUserSession>> {
		self.user_session.1.try_recv().ok()
	}

	pub fn try_recv_images(&self) -> TryIter<ImageMsg> {
		self.image.1.try_iter()
	}

	pub fn try_recv_user(&self) -> Option<NeosUser> {
		self.user.1.try_recv().ok()
	}
	pub fn try_recv_user_status(&self) -> Option<UserStatusMsg> {
		self.user_status.1.try_recv().ok()
	}
	pub fn try_recv_session(&self) -> Option<NeosSession> {
		self.session.1.try_recv().ok()
	}
}

impl NeosPeepsApp {
	/// Tries to receive messages from other threads
	pub fn try_recv(&mut self, frame: &epi::Frame) {
		let mut repaint = false;

		if let Some(friends) = self.channels.try_recv_friends() {
			self.runtime.friends = friends;
			self.runtime.loading.fetching_friends = false;
			repaint = true;
		}

		if let Some(users) = self.channels.try_recv_users() {
			self.runtime.users = users;
			self.runtime.loading.fetching_users = false;
			repaint = true;
		}

		if let Some(sessions) = self.channels.try_recv_sessions() {
			self.runtime.sessions = sessions;
			self.runtime.loading.fetching_sessions = false;
			repaint = true;
		}

		if let Some(user_session) = self.channels.try_recv_user_session() {
			self.stored.user_session = user_session;
			*self.runtime.session_window.borrow_mut() = None;
			*self.runtime.user_window.borrow_mut() = None;
		}

		if let Some(client) = self.channels.try_recv_auth() {
			self.runtime.neos_api = client;
			self.runtime.loading.login = LoginOperationState::None;
			repaint = true;
		}

		for (id, image) in self.channels.try_recv_images() {
			self.runtime.loading_textures.get_mut().remove(&id);
			if let Some(image) = image {
				self.runtime.textures.insert(id, Rc::new(image));
			}
			repaint = true;
		}

		if let Some(user) = self.channels.try_recv_user() {
			if let Some((user_id, w_user, _)) =
				&mut *self.runtime.user_window.borrow_mut()
			{
				if user.id == *user_id {
					*w_user = Some(user);
				}
			}
			repaint = true;
		}

		if let Some((user_id, user_status)) =
			self.channels.try_recv_user_status()
		{
			if let Some((w_user_id, _, w_user_status)) =
				&mut *self.runtime.user_window.borrow_mut()
			{
				if user_id == *w_user_id {
					*w_user_status = Some(user_status);
				}
			}
			repaint = true;
		}

		if let Some(session) = self.channels.try_recv_session() {
			if let Some((session_id, w_session)) =
				&mut *self.runtime.session_window.borrow_mut()
			{
				if session.session_id == *session_id {
					*w_session = Some(session);
				}
			}
			repaint = true;
		}

		if repaint {
			frame.request_repaint();
		}
	}
}
