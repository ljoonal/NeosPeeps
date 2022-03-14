//! The friends page of the app

use std::collections::HashMap;

use ahash::RandomState;
use eframe::epi;
use neos::api_client::AnyNeos;

use crate::app::NeosPeepsApp;

#[allow(clippy::module_name_repetitions)]
pub type UserMessages = HashMap<String, neos::Message, RandomState>;
#[allow(clippy::module_name_repetitions)]
pub type AllMessages = HashMap<neos::id::User, UserMessages, RandomState>;

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
			if let AnyNeos::Authenticated(neos_api) = &*neos_api_arc {
				match neos_api.get_messages(100, false, None, None) {
					Ok(messages) => {
						messages_sender
							.send(Ok(Self::sort_all_messages(messages)))
							.unwrap();
					}
					Err(e) => {
						messages_sender.send(Err(e.to_string())).unwrap();
					}
				}
			}
		});

		frame.request_repaint();
	}

	fn sort_all_messages(messages: Vec<neos::Message>) -> AllMessages {
		let hash_builder = RandomState::new();
		let mut sorted_messages: AllMessages = HashMap::with_hasher(hash_builder);

		// TODO: if this is too slow switch to rayon's par iter
		for (message_id, message) in Self::sort_user_messages(messages) {
			let non_owner_id = message.non_owner_id().clone();

			if let Some(map) = sorted_messages.get_mut(&non_owner_id) {
				map.insert(message_id, message);
			} else {
				let hash_builder = RandomState::new();
				let mut map: UserMessages = HashMap::with_hasher(hash_builder);
				map.insert(message_id, message);
				sorted_messages.insert(non_owner_id, map);
			}
		}

		sorted_messages
	}

	fn sort_user_messages(messages: Vec<neos::Message>) -> UserMessages {
		use rayon::prelude::*;
		messages
			.into_par_iter()
			.map(|message| (message.id.clone(), message))
			.collect()
	}
}
