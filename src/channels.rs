use std::sync::Arc;

use crossbeam::channel::{bounded, Receiver, Sender};
use eframe::epi;
use neos::{api_client::AnyNeos, NeosFriend, NeosSession};

use crate::app::NeosPeepsApp;

pub struct Channels {
	friends: (Sender<Vec<NeosFriend>>, Receiver<Vec<NeosFriend>>),
	sessions: (Sender<Vec<NeosSession>>, Receiver<Vec<NeosSession>>),
	auth: (Sender<AnyNeos>, Receiver<AnyNeos>),
}

impl Default for Channels {
	fn default() -> Self {
		Self { friends: bounded(1), auth: bounded(1), sessions: bounded(1) }
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
	pub fn auth_sender(&self) -> Sender<AnyNeos> {
		self.auth.0.clone()
	}
	pub fn try_recv_friends(&self) -> Option<Vec<NeosFriend>> {
		self.friends.1.try_recv().ok()
	}

	pub fn try_recv_sessions(&self) -> Option<Vec<NeosSession>> {
		self.sessions.1.try_recv().ok()
	}

	pub fn try_recv_auth(&self) -> Option<AnyNeos> {
		self.auth.1.try_recv().ok()
	}
}

impl NeosPeepsApp {
	/// Tries to receive messages from other threads
	pub fn try_recv(&mut self, frame: &epi::Frame) {
		if let Some(friends) = self.channels.try_recv_friends() {
			//
		}

		if let Some(sessions) = self.channels.try_recv_sessions() {
			//
		}

		if let Some(client) = self.channels.try_recv_auth() {
			self.runtime.neos_api = Arc::new(client);
		}
	}
}
