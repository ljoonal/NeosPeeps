use std::cell::RefCell;

#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Default)]
pub struct Tracker {
	pub friends: RefCell<bool>,
	pub users: RefCell<bool>,
	pub sessions: RefCell<bool>,
	pub user: RefCell<bool>,
	pub user_status: RefCell<bool>,
	pub session: RefCell<bool>,
}

impl Tracker {
	fn any(&self) -> bool {
		*self.friends.borrow()
			|| *self.users.borrow()
			|| *self.sessions.borrow()
			|| *self.user.borrow()
			|| *self.user_status.borrow()
			|| *self.session.borrow()
	}
}
