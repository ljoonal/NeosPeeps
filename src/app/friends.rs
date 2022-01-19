//! The friends page of the app

use super::{sessions::find_focused_session, NeosPeepsApp};
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
	/// Refreshes friends in a background thread
	pub fn refresh_friends(&mut self, frame: &epi::Frame) {
		if self.runtime.loading.login_op() {
			return;
		}
		self.runtime.loading.fetching_friends = true;

		frame.request_repaint();

		let neos_api_arc = self.runtime.neos_api.clone();
		let friends_sender = self.channels.friends_sender();
		rayon::spawn(move || {
			if let AnyNeos::Authenticated(neos_api) = &*neos_api_arc {
				match neos_api.get_friends() {
					Ok(mut friends) => {
						friends.sort_by(order_friends);
						if let Err(err) = friends_sender.send(friends) {
							println!(
								"Failed to send friends to main thread! {}",
								err
							);
						}
					}
					Err(e) => {
						println!("Error with Neos API: {}", e);
					}
				}
			}
		});
	}

	fn friend_row(
		&self,
		ui: &mut Ui,
		width: f32,
		frame: &epi::Frame,
		friend: &NeosFriend,
	) {
		ui.with_layout(Layout::left_to_right(), |ui| {
			let pfp_url: &Option<AssetUrl> = get_pfp_url(friend);

			let pfp = match pfp_url {
				Some(pfp_url) => self.load_texture(pfp_url, frame),
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
			ui.set_max_width((width - self.stored.row_height) / 2_f32);

			ui.separator();
			ui.vertical(|ui| {
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
			ui.set_min_width(ui.available_width());
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
						let session_pics = self.load_texture(thumbnail, frame);

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

	pub fn friends_page(&mut self, ui: &mut Ui, frame: &epi::Frame) {
		let friends_count = self.runtime.friends.len();

		ui.heading(friends_count.to_string() + " Peeps");

		ScrollArea::both().show_rows(
			ui,
			self.stored.row_height,
			friends_count,
			|ui, row_range| {
				let width = ui.available_width();
				Grid::new("friends_list")
					.start_row(row_range.start)
					.min_row_height(self.stored.row_height)
					.num_columns(3)
					.show(ui, |ui| {
						for row in row_range {
							let friend = &self.runtime.friends[row];
							self.friend_row(ui, width, frame, friend);
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
