//! The friends page of the app

use super::{sessions::find_focused_session, NeosPeepsApp};
use crate::image::TextureDetails;
use eframe::{
	egui::{Color32, Grid, Label, Layout, RichText, ScrollArea, Ui, Vec2},
	epi,
};
use neos::{api_client::AnyNeos, AssetUrl, NeosFriend, NeosUserOnlineStatus};
use std::cmp::Ordering;

fn order_friends(fren1: &NeosFriend, fren2: &NeosFriend) -> Ordering {
	// if their current session is joinable
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

	// if the friends are marked as online
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

	// if at least not offline
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
	Ordering::Equal
}

impl NeosPeepsApp {
	fn friend_row(&self, ui: &mut Ui, frame: &epi::Frame, friend: &NeosFriend) {
		ui.with_layout(Layout::left_to_right(), |ui| {
			let pfp_url: &Option<AssetUrl> = get_pfp_url(friend);

			let pfp = match pfp_url {
				Some(pfp_url) => self.runtime.load_texture(pfp_url, frame),
				None => None,
			};

			let pfp = pfp.unwrap_or_else(|| {
				self.runtime.default_profile_picture.clone().unwrap()
			});

			ui.image(
				pfp.id,
				Vec2::new(self.stored.row_height, self.stored.row_height),
			);
		});

		ui.with_layout(Layout::left_to_right(), |ui| {
			ui.separator();
			ui.vertical(|ui| {
				ui.set_max_width(self.stored.row_height * 2_f32);
				let (r, g, b) = friend.user_status.online_status.color();
				ui.heading(&friend.friend_username);
				ui.label(
					RichText::new(
						&friend.user_status.online_status.to_string(),
					)
					.color(Color32::from_rgb(r, g, b)),
				);
				ui.label(RichText::new(friend.id.as_ref()).small().monospace());
			});
		});

		ui.with_layout(Layout::left_to_right(), |ui| {
			ui.separator();

			let session = find_focused_session(&friend.id, &friend.user_status);

			ui.vertical(|ui| {
				if let Some(session) = session {
					ui.add(Label::new(&session.name).wrap(true));
					ui.label(
						friend
							.user_status
							.current_session_access_level
							.as_ref(),
					);
					ui.label(&format!(
						"{}/{}",
						&session.joined_users, &session.max_users
					));
				} else if friend.user_status.online_status
					== NeosUserOnlineStatus::Offline
				{
					ui.label(friend.user_status.online_status.as_ref());
				} else {
					ui.label("Couldn't find focused session");
					ui.label(
						friend
							.user_status
							.current_session_access_level
							.as_ref(),
					);
				}
			});

			ui.vertical(|ui| {
				if let Some(session) = session {
					if let Some(thumbnail) = &session.thumbnail {
						let session_pics =
							self.runtime.load_texture(thumbnail, frame);

						if let Some(session_pic) = session_pics {
							let scaling =
								ui.available_height() / session_pic.size.y;
							ui.image(
								session_pic.id,
								session_pic.size * scaling,
							);
						}
					}
				}
			});
		});

		ui.end_row();
	}

	/// Refreshes friends in a background thread
	pub fn refresh_friends(&mut self, frame: epi::Frame) {
		{
			let mut loading = self.runtime.loading.write().unwrap();
			if loading.is_loading() {
				return;
			}
			*loading = crate::data::LoadingState::FetchingFriends;
		}
		frame.request_repaint();

		let neos_api_arc = self.runtime.neos_api.clone();
		let friends_arc = self.runtime.friends.clone();
		let loading = self.runtime.loading.clone();
		rayon::spawn(move || {
			if let AnyNeos::Authenticated(neos_api) =
				&*neos_api_arc.read().unwrap()
			{
				match neos_api.get_friends() {
					Ok(mut friends) => {
						friends.sort_by(order_friends);
						*friends_arc.write().unwrap() = friends;
					}
					Err(e) => {
						println!("Error with Neos API: {}", e);
					}
				}
			}

			*loading.write().unwrap() = crate::data::LoadingState::None;
			frame.request_repaint();
		});
	}

	pub fn friends_page(&mut self, ui: &mut Ui, frame: &epi::Frame) {
		ui.heading(&("Peeps of ".to_owned() + self.stored.identifier.inner()));

		let friends = self.runtime.friends.read().unwrap();

		ScrollArea::both().show_rows(
			ui,
			self.stored.row_height,
			friends.len(),
			|ui, row_range| {
				ui.set_width(ui.available_width());
				Grid::new("friends_list")
					.start_row(row_range.start)
					.min_col_width(self.stored.row_height)
					.num_columns(3)
					.show(ui, |ui| {
						ui.set_height(self.stored.row_height);
						ui.set_width(ui.available_width());
						for row in row_range {
							let friend = &friends[row];
							self.friend_row(ui, frame, friend);
						}
					});
			},
		);
	}
}
pub const fn get_pfp_url(friend: &NeosFriend) -> &Option<AssetUrl> {
	match &friend.profile {
		Some(profile) => &profile.icon_url,
		None => &None,
	}
}
