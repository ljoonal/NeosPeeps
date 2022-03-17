//! The friends page of the app

use std::{borrow::Borrow, cmp::Ordering, collections::HashMap, sync::Arc};

use ahash::RandomState;
use chrono::{DateTime, Utc};
use crossbeam::channel::Sender;
use eframe::epi;
use neos::api_client::AnyNeos;

use crate::app::NeosPeepsApp;

#[allow(clippy::module_name_repetitions)]
pub type UserMessages = sorted_vec::SortedSet<Message>;
#[allow(clippy::module_name_repetitions)]
pub type AllMessages = HashMap<neos::id::User, UserMessages, RandomState>;

#[derive(Debug)]
pub struct Message(pub neos::Message);

impl PartialEq for Message {
	fn eq(&self, other: &Self) -> bool { self.0.id == other.0.id }
}
impl Eq for Message {}

impl Ord for Message {
	fn cmp(&self, other: &Self) -> Ordering {
		self.0.send_time.cmp(&other.0.send_time)
	}
}

impl PartialOrd for Message {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}

impl NeosPeepsApp {
	/// Refreshes messages in a background thread
	pub fn refresh_messages(&mut self, frame: &epi::Frame) {
		let neos_api_arc = match &self.runtime.neos_api {
			Some(api) => api.clone(),
			None => return,
		};

		self.threads.loading.messages.set(true);
		let messages_sender = self.threads.channels.messages_sender();
		self.threads.spawn_data_op(move || {
			Self::get_messages(neos_api_arc, messages_sender, 100, true, None, None);
		});

		frame.request_repaint();
	}

	pub fn fetch_user_chat(
		&mut self, frame: &epi::Frame, user: neos::id::User,
		from_time: Option<DateTime<Utc>>,
	) {
		let neos_api_arc = match &self.runtime.neos_api {
			Some(api) => api.clone(),
			None => return,
		};

		self.threads.loading.messages.set(true);
		let messages_sender = self.threads.channels.messages_sender();
		self.threads.spawn_data_op(move || {
			Self::get_messages(
				neos_api_arc,
				messages_sender,
				100,
				false,
				from_time,
				Some(user),
			);
		});

		frame.request_repaint();
	}

	#[allow(clippy::needless_pass_by_value)]
	fn get_messages(
		neos_api_arc: Arc<AnyNeos>,
		messages_sender: Sender<Result<AllMessages, String>>, max_amount: u16,
		unread_only: bool, from_time: impl Borrow<Option<DateTime<Utc>>>,
		user: impl Borrow<Option<neos::id::User>>,
	) {
		if let AnyNeos::Authenticated(neos_api) = &*neos_api_arc {
			match neos_api.get_messages(max_amount, unread_only, from_time, user) {
				Ok(messages) => {
					messages_sender.send(Ok(Self::split_by_user(messages))).unwrap();
				}
				Err(e) => {
					messages_sender.send(Err(e.to_string())).unwrap();
				}
			}
		}
	}

	pub fn send_message(&mut self, frame: &epi::Frame, message: neos::Message) {
		let neos_api_arc = match &self.runtime.neos_api {
			Some(api) => api.clone(),
			None => return,
		};

		self.threads.loading.messages.set(true);
		let messages_sender = self.threads.channels.messages_sender();
		self.threads.spawn_data_op(move || {
			if let AnyNeos::Authenticated(neos_api) = &*neos_api_arc {
				let to_id = message.recipient_id.clone();
				if let Err(e) = neos_api.send_message(message) {
					eprintln!("Error sending message! {:?}", e);
				}
				Self::get_messages(
					neos_api_arc,
					messages_sender,
					32,
					false,
					None,
					Some(to_id),
				);
			}
		});

		frame.request_repaint();
	}

	fn split_by_user(messages: Vec<neos::Message>) -> AllMessages {
		let hash_builder = RandomState::new();
		let mut sorted_messages: AllMessages = HashMap::with_hasher(hash_builder);

		// TODO: if this is too slow switch to rayon's par iter
		for message in messages {
			let non_owner_id = message.non_owner_id().clone();

			if let Some(set) = sorted_messages.get_mut(&non_owner_id) {
				set.insert(Message(message));
			} else {
				let mut set: UserMessages = sorted_vec::SortedSet::new();
				set.insert(Message(message));
				sorted_messages.insert(non_owner_id, set);
			}
		}

		sorted_messages
	}
}
