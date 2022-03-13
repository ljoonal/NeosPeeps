use std::sync::Arc;

use crossbeam::channel::{unbounded, Receiver, Sender, TryIter};
use eframe::egui::TextureHandle;
use neos::api_client::AnyNeos;

use crate::updating::GiteaReleasesResponse;

type ImageMsg = (String, Option<TextureHandle>);
type UserStatusMsg = (neos::id::User, neos::UserStatus);

// Sender & Receiver than can have errors.
type Res<T> = Result<T, String>;
type ResSender<T> = Sender<Res<T>>;
type ResReceiver<T> = Receiver<Res<T>>;

#[derive(Debug)]
pub struct Channels {
	/// Friends bg refresh
	friends: (ResSender<Vec<neos::Friend>>, ResReceiver<Vec<neos::Friend>>),
	/// Users search
	users: (ResSender<Vec<neos::User>>, ResReceiver<Vec<neos::User>>),
	/// Sessions bg refresh
	sessions:
		(ResSender<Vec<neos::SessionInfo>>, ResReceiver<Vec<neos::SessionInfo>>),
	/// Login/Logout
	auth: (Sender<Arc<AnyNeos>>, Receiver<Arc<AnyNeos>>),
	/// New login was successful
	user_session:
		(Sender<Option<neos::UserSession>>, Receiver<Option<neos::UserSession>>),
	/// Image assets being loaded
	image: (Sender<ImageMsg>, Receiver<ImageMsg>),
	/// Lookups for the user window
	user: (ResSender<neos::User>, ResReceiver<neos::User>),
	/// Lookups for the user window
	user_status: (ResSender<UserStatusMsg>, ResReceiver<UserStatusMsg>),
	/// Lookups for the session window
	session: (ResSender<neos::SessionInfo>, ResReceiver<neos::SessionInfo>),
	update_check:
		(Sender<GiteaReleasesResponse>, Receiver<GiteaReleasesResponse>),
}

impl Default for Channels {
	fn default() -> Self {
		Self {
			friends: unbounded(),
			users: unbounded(),
			auth: unbounded(),
			sessions: unbounded(),
			user_session: unbounded(),
			image: unbounded(),
			user: unbounded(),
			user_status: unbounded(),
			session: unbounded(),
			update_check: unbounded(),
		}
	}
}

// Allow trying to receive or getting a sender but nothing more.
impl Channels {
	pub fn friends_sender(&self) -> ResSender<Vec<neos::Friend>> {
		self.friends.0.clone()
	}

	pub fn users_sender(&self) -> ResSender<Vec<neos::User>> {
		self.users.0.clone()
	}

	pub fn sessions_sender(&self) -> ResSender<Vec<neos::SessionInfo>> {
		self.sessions.0.clone()
	}

	pub fn auth_sender(&self) -> Sender<Arc<AnyNeos>> { self.auth.0.clone() }

	pub fn user_session_sender(&self) -> Sender<Option<neos::UserSession>> {
		self.user_session.0.clone()
	}

	pub fn image_sender(&self) -> Sender<ImageMsg> { self.image.0.clone() }

	pub fn user_sender(&self) -> ResSender<neos::User> { self.user.0.clone() }

	pub fn user_status_sender(&self) -> ResSender<UserStatusMsg> {
		self.user_status.0.clone()
	}

	pub fn session_sender(&self) -> ResSender<neos::SessionInfo> {
		self.session.0.clone()
	}

	pub fn update_check_sender(&self) -> Sender<GiteaReleasesResponse> {
		self.update_check.0.clone()
	}

	pub fn try_recv_friends(&self) -> Option<Res<Vec<neos::Friend>>> {
		self.friends.1.try_recv().ok()
	}

	pub fn try_recv_users(&self) -> Option<Res<Vec<neos::User>>> {
		self.users.1.try_recv().ok()
	}

	pub fn try_recv_sessions(&self) -> Option<Res<Vec<neos::SessionInfo>>> {
		self.sessions.1.try_recv().ok()
	}

	pub fn try_recv_auth(&self) -> Option<Arc<AnyNeos>> {
		self.auth.1.try_recv().ok()
	}

	#[allow(clippy::option_option)]
	pub fn try_recv_user_session(&self) -> Option<Option<neos::UserSession>> {
		self.user_session.1.try_recv().ok()
	}

	pub fn try_recv_images(&self) -> TryIter<ImageMsg> { self.image.1.try_iter() }

	pub fn try_recv_user(&self) -> Option<Res<neos::User>> {
		self.user.1.try_recv().ok()
	}

	pub fn try_recv_user_status(&self) -> Option<Res<UserStatusMsg>> {
		self.user_status.1.try_recv().ok()
	}

	pub fn try_recv_session(&self) -> Option<Res<neos::SessionInfo>> {
		self.session.1.try_recv().ok()
	}

	pub fn try_recv_updates(&self) -> Option<GiteaReleasesResponse> {
		self.update_check.1.try_recv().ok()
	}
}
