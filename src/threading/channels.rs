use std::sync::Arc;

use crossbeam::channel::{unbounded, Receiver, Sender, TryIter};
use neos::{
	api_client::AnyNeos,
	NeosFriend,
	NeosSession,
	NeosUser,
	NeosUserSession,
	NeosUserStatus,
};

use crate::image::TextureDetails;

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
			friends: unbounded(),
			users: unbounded(),
			auth: unbounded(),
			sessions: unbounded(),
			user_session: unbounded(),
			image: unbounded(),
			user: unbounded(),
			user_status: unbounded(),
			session: unbounded(),
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
