//! The friends page of the app

use std::{
	cmp::Ordering,
	collections::HashMap,
	sync::{Arc, RwLock},
	time::Instant,
};

use crate::image::TextureDetails;
use neos::{api_client::AnyNeos, AssetUrl, NeosFriend, NeosUserOnlineStatus};

use super::NeosPeepsApp;
use eframe::{
	egui::{Color32, Grid, Label, Layout, RichText, ScrollArea, Ui, Vec2},
	epi,
};

const ROW_HEIGHT: f32 = 128_f32;

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
	fn friend_row(
		&self,
		ui: &mut Ui,
		friend: &NeosFriend,
		pfp: &TextureDetails,
	) {
		ui.with_layout(Layout::left_to_right(), |ui| {
			ui.image(pfp.id, Vec2::new(ROW_HEIGHT, ROW_HEIGHT));
		});

		ui.with_layout(Layout::left_to_right(), |ui| {
			ui.separator();
			ui.vertical(|ui| {
				ui.set_max_width(ROW_HEIGHT * 2_f32);
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
			ui.vertical(|ui| {
				if friend.user_status.online_status
					== NeosUserOnlineStatus::Offline
				{
					ui.add(Label::new("Offline").wrap(false));
				} else {
					ui.add(
						Label::new("Current session access level").wrap(false),
					);
					ui.label(
						friend
							.user_status
							.current_session_access_level
							.as_ref(),
					);
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
		let friend_pics = self.runtime.friend_pics.clone();
		let last_friends_refresh = self.runtime.last_friends_refresh.clone();
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

			*last_friends_refresh.write().unwrap() = Instant::now();
			*loading.write().unwrap() = crate::data::LoadingState::None;
			frame.request_repaint();

			unload_unused_friend_pics(
				&mut *friend_pics.write().unwrap(),
				&*friends_arc.read().unwrap(),
			);
		});
	}

	pub fn friends_page(&mut self, ui: &mut Ui, frame: &epi::Frame) {
		ui.heading(&("Peeps of ".to_owned() + self.stored.identifier.inner()));
		if self.runtime.loading.read().unwrap().is_loading() {
			ui.label("Loading...");
		}

		if self.runtime.default_profile_picture.is_none() {
			let user_img = image::load_from_memory(include_bytes!(
				"../../static/user.png"
			))
			.expect("Failed to load image");
			self.runtime.default_profile_picture =
				Some(TextureDetails::from_image(frame.clone(), &user_img));
		}
		let default_profile_pic =
			self.runtime.default_profile_picture.as_ref().unwrap();

		let friends = self.runtime.friends.read().unwrap();

		ScrollArea::both().show_rows(
			ui,
			ROW_HEIGHT,
			friends.len(),
			|ui, row_range| {
				ui.set_width(ui.available_width());
				Grid::new("friends_list")
					.start_row(row_range.start)
					.min_col_width(ROW_HEIGHT)
					.num_columns(3)
					.show(ui, |ui| {
						ui.set_height(ROW_HEIGHT);
						ui.set_width(ui.available_width());
						for row in row_range {
							let friend = &friends[row];

							let pfp_url: Option<&AssetUrl> =
								match get_pfp_url(friend) {
									Some(asset_url) => {
										load_friend_pic(
											asset_url,
											self.runtime.friend_pics.clone(),
											frame,
										);
										Some(asset_url)
									}
									None => None,
								};

							let friend_pics =
								self.runtime.friend_pics.read().unwrap();

							// Gets the profile picture from URL.
							let pfp: &TextureDetails = match pfp_url {
								Some(pfp_url) => {
									match friend_pics.get(pfp_url.id()) {
										Some(Some(pfp)) => pfp,
										_ => default_profile_pic,
									}
								}
								None => default_profile_pic,
							};

							self.friend_row(ui, friend, pfp);
						}
					});
			},
		);
	}
}

fn load_friend_pic(
	asset_url: &neos::AssetUrl,
	pics: Arc<RwLock<HashMap<String, Option<TextureDetails>>>>,
	frame: &epi::Frame,
) {
	{
		let mut pics = pics.write().unwrap();
		if pics.contains_key(asset_url.id()) {
			return;
		}
		pics.insert(asset_url.id().to_owned(), None);
	}

	let asset_url = asset_url.clone();
	let frame = frame.clone();
	rayon::spawn(move || match crate::image::get(&asset_url) {
		Ok(image) => {
			let (size, image) = crate::image::to_epi_format(&image);
			pics.write().unwrap().insert(
				asset_url.id().to_owned(),
				Some(TextureDetails::new(frame.clone(), size, image)),
			);
			frame.request_repaint();
		}
		Err(err) => {
			println!(
				"Failed to fetch the profile picture `{}`: {}",
				asset_url.id(),
				err
			);
		}
	});
}

fn unload_unused_friend_pics(
	pics: &mut HashMap<String, Option<TextureDetails>>,
	friends: &[NeosFriend],
) {
	use rayon::prelude::*;
	pics.retain(|id, pic| {
		pic.is_none() // Prevent multiple fetches.
			|| friends
				.par_iter()
				.find_any(|friend| match get_pfp_url(friend) {
					Some(pfp_url) => pfp_url.id() == *id,
					None => false,
				})
				.is_some()
	});
}

const fn get_pfp_url(friend: &NeosFriend) -> &Option<AssetUrl> {
	match &friend.profile {
		Some(profile) => &profile.icon_url,
		None => &None,
	}
}
