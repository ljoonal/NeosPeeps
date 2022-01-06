//! The friends page of the app

use std::cmp::Ordering;

use crate::image::ImageDetails;
use neos::{NeosFriend, NeosUserOnlineStatus};

use super::NeosPeepsApp;
use eframe::{
	egui::{Color32, RichText, Ui, Vec2},
	epi,
};

fn order_friends(fren1: &NeosFriend, fren2: &NeosFriend) -> Ordering {
	// First sort on if the friends are marked as online
	if fren1.user_status.online_status == NeosUserOnlineStatus::Online
		&& fren2.user_status.online_status != NeosUserOnlineStatus::Online
	{
		return Ordering::Less;
	};
	if fren1.user_status.online_status != NeosUserOnlineStatus::Online
		&& fren2.user_status.online_status == NeosUserOnlineStatus::Online
	{
		return Ordering::Greater;
	};

	// Then if at least not offline
	if fren1.user_status.online_status != NeosUserOnlineStatus::Offline
		&& fren2.user_status.online_status == NeosUserOnlineStatus::Offline
	{
		return Ordering::Less;
	};
	if fren1.user_status.online_status == NeosUserOnlineStatus::Offline
		&& fren2.user_status.online_status != NeosUserOnlineStatus::Offline
	{
		return Ordering::Greater;
	};

	// Then if their current session is joinable
	if fren1.user_status.current_session_access_level
		> fren2.user_status.current_session_access_level
	{
		return Ordering::Less;
	};
	if fren1.user_status.current_session_access_level
		< fren2.user_status.current_session_access_level
	{
		return Ordering::Greater;
	};
	Ordering::Equal
}

impl NeosPeepsApp {
	fn friend_card(&self, ui: &mut Ui, friend: &NeosFriend) {
		const CARD_HEIGHT: f32 = 128f32;

		ui.horizontal(|ui| {
			// Should never be None, but let's be safe...
			if let Some(pfp) = self.default_profile_picture.as_ref() {
				ui.image(pfp.id, Vec2::new(CARD_HEIGHT, CARD_HEIGHT));

				ui.separator();
			}

			ui.vertical(|ui| {
				let (r, g, b) = friend.user_status.online_status.color();
				ui.heading(&friend.friend_username);
				ui.label(
					RichText::new(
						&friend.user_status.online_status.to_string(),
					)
					.color(Color32::from_rgb(r, g, b)),
				);
				ui.label(RichText::new(&friend.id).small().monospace());
			});

			ui.separator();

			ui.vertical(|ui| {
				ui.label("Not in a world");
			});
		});
	}

	/// Refreshes friends in a background thread
	pub fn refresh_friends(&mut self, frame: epi::Frame) {
		let friends_arc = self.friends.clone();
		let neos_api = self.neos_api.clone();
		let loading = self.loading_data.clone();
		std::thread::spawn(move || {
			*loading.write().unwrap() = true;
			frame.request_repaint();

			if let Some(neos_api) = &*neos_api.read().unwrap() {
				match neos_api.fetch_friends() {
					Ok(mut friends) => {
						friends.sort_by(order_friends);
						*friends_arc.write().unwrap() = friends;
					}
					Err(e) => {
						println!("Error with Neos API: {}", e);
					}
				}
			}

			*loading.write().unwrap() = false;
			frame.request_repaint();
		});
	}

	pub fn friends_page(&mut self, ui: &mut Ui, frame: &epi::Frame) {
		ui.heading(&("Peeps of ".to_owned() + &self.username));
		if *self.loading_data.read().unwrap() {
			ui.label("Loading...");
		}

		if self.default_profile_picture.is_none() {
			let user_img = image::load_from_memory(include_bytes!(
				"../../static/user.png"
			))
			.expect("Failed to load image");
			self.default_profile_picture =
				Some(ImageDetails::from_image(frame.clone(), user_img));
		}

		for friend in self.friends.read().unwrap().iter() {
			ui.separator();
			self.friend_card(ui, friend);
		}
	}
}
