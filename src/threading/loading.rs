use std::cell::Cell;

#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Default)]
pub struct Tracker {
	pub friends: Cell<bool>,
	pub users: Cell<bool>,
	pub sessions: Cell<bool>,
	pub user: Cell<bool>,
	pub user_status: Cell<bool>,
	pub session: Cell<bool>,
}

impl Tracker {
	pub fn any(&self) -> bool {
		self.friends.get()
			|| self.users.get()
			|| self.sessions.get()
			|| self.user.get()
			|| self.user_status.get()
			|| self.session.get()
	}
}
