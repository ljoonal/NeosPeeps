use std::{rc::Rc, sync::Arc};

use crossbeam::channel::{bounded, unbounded, Receiver, Sender, TryIter};
use eframe::epi;
use neos::{api_client::AnyNeos, NeosFriend, NeosSession, NeosUserSession};

use crate::{
	app::NeosPeepsApp, data::LoginOperationState, image::TextureDetails,
};

type ImageMsg = (String, Option<TextureDetails>);

pub struct Channels {
	friends: (Sender<Vec<NeosFriend>>, Receiver<Vec<NeosFriend>>),
	sessions: (Sender<Vec<NeosSession>>, Receiver<Vec<NeosSession>>),
	auth: (Sender<Arc<AnyNeos>>, Receiver<Arc<AnyNeos>>),
	user_session:
		(Sender<Option<NeosUserSession>>, Receiver<Option<NeosUserSession>>),
	image: (Sender<ImageMsg>, Receiver<ImageMsg>),
}

impl Default for Channels {
	fn default() -> Self {
		Self {
			friends: bounded(1),
			auth: bounded(1),
			sessions: bounded(1),
			user_session: bounded(1),
			image: unbounded(),
		}
	}
}

// Allow trying to receive or getting a sender but nothing more.
impl Channels {
	pub fn friends_sender(&self) -> Sender<Vec<NeosFriend>> {
		self.friends.0.clone()
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

	pub fn try_recv_friends(&self) -> Option<Vec<NeosFriend>> {
		self.friends.1.try_recv().ok()
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

		if let Some(sessions) = self.channels.try_recv_sessions() {
			self.runtime.sessions = sessions;
			self.runtime.loading.fetching_sessions = false;
			repaint = true;
		}

		if let Some(user_session) = self.channels.try_recv_user_session() {
			self.stored.user_session = user_session;
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
		}

		if repaint {
			frame.request_repaint();
		}
	}
}
