//! The friends page of the app

use super::{sessions::find_focused_session, NeosPeepsApp};
use eframe::{
	egui::{
		Color32, CtxRef, Grid, Key, Label, Layout, RichText, ScrollArea, Ui,
		Vec2, Window,
	},
	epi,
};
use neos::{
	api_client::{AnyNeos, Neos, UserIdOrUsername},
	AssetUrl, NeosFriend, NeosSession, NeosUser, NeosUserOnlineStatus,
	NeosUserStatus,
};
use std::cmp::Ordering;

fn order_users(s1: &NeosUserStatus, s2: &NeosUserStatus) -> Ordering {
	// if their current session is joinable
	if s1.current_session_access_level > s2.current_session_access_level {
		return Ordering::Less;
	};
	if s1.current_session_access_level < s2.current_session_access_level {
		return Ordering::Greater;
	};

	// if the friends are marked as online
	if s1.online_status == NeosUserOnlineStatus::Online
		&& s2.online_status != NeosUserOnlineStatus::Online
	{
		return Ordering::Less;
	};
	if s1.online_status != NeosUserOnlineStatus::Online
		&& s2.online_status == NeosUserOnlineStatus::Online
	{
		return Ordering::Greater;
	};

	// if at least not offline
	if s1.online_status != NeosUserOnlineStatus::Offline
		&& s2.online_status == NeosUserOnlineStatus::Offline
	{
		return Ordering::Less;
	};
	if s1.online_status == NeosUserOnlineStatus::Offline
		&& s2.online_status != NeosUserOnlineStatus::Offline
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
						friends.sort_by(|f1, f2| {
							order_users(&f1.user_status, &f2.user_status)
						});
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

	pub fn search_users(&mut self, frame: &epi::Frame) {
		if self.stored.filter_search.is_empty()
			|| self.runtime.loading.login_op()
		{
			return;
		}

		frame.request_repaint();

		let neos_api = self.runtime.neos_api.clone();
		let users_sender = self.channels.users_sender();
		let search = self.stored.filter_search.clone();
		rayon::spawn(move || match neos_api.search_users(&search) {
			Ok(users) => {
				if let Err(err) = users_sender.send(users) {
					println!("Failed to send users to main thread! {}", err);
				}
			}
			Err(e) => {
				println!("Error with Neos API: {}", e);
			}
		});
	}

	/// Gets the user for the user window
	pub fn get_user(&mut self, frame: &epi::Frame, id: neos::id::User) {
		if self.stored.filter_search.is_empty()
			|| self.runtime.loading.login_op()
		{
			return;
		}
		if let Some((w_id, _, _)) = &self.runtime.user_window {
			if w_id != &id {
				return;
			}
		} else {
			self.runtime.user_window = Some((id.clone(), None, None));
		}

		frame.request_repaint();

		let neos_api = self.runtime.neos_api.clone();
		let user_sender = self.channels.user_sender();
		rayon::spawn(move || match neos_api.get_user(id) {
			Ok(user) => {
				if let Err(err) = user_sender.send(user) {
					println!("Failed to send user to main thread! {}", err);
				}
			}
			Err(e) => {
				println!("Error with Neos API: {}", e);
			}
		});
	}

	/// Gets the user status for the user window
	pub fn get_user_status(&mut self, frame: &epi::Frame, id: neos::id::User) {
		if self.stored.filter_search.is_empty()
			|| self.runtime.loading.login_op()
		{
			return;
		}
		if let Some((w_id, _, _)) = &self.runtime.user_window {
			if w_id != &id {
				return;
			}
		} else {
			self.runtime.user_window = Some((id.clone(), None, None));
		}

		frame.request_repaint();

		let neos_api = self.runtime.neos_api.clone();
		let user_status_sender = self.channels.user_status_sender();
		rayon::spawn(move || match neos_api.get_user_status(id.clone()) {
			Ok(user_status) => {
				if let Err(err) = user_status_sender.send((id, user_status)) {
					println!(
						"Failed to send user status to main thread! {}",
						err
					);
				}
			}
			Err(e) => {
				println!("Error with Neos API: {}", e);
			}
		});
	}

	pub fn user_window(&mut self, ctx: &CtxRef, frame: &epi::Frame) {
		Window::new("User").show(ctx, |ui| {
			if ui.button("Close").clicked() {
				self.runtime.user_window = None;
				//...
			}
		});
	}

	fn if_four_col(&self, width: f32) -> bool {
		const COL_MIN_WIDTH: f32 = 300.;

		(width - self.stored.row_height * 2_f32) / 2_f32 > COL_MIN_WIDTH
	}

	fn friend_row(
		&self,
		ui: &mut Ui,
		width: f32,
		frame: &epi::Frame,
		friend: &NeosFriend,
	) {
		ui.with_layout(Layout::left_to_right(), |ui| {
			let pfp_url: &Option<AssetUrl> = get_friend_pfp(friend);

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

		let style = ui.style();

		let width_for_cols = self.stored.row_height.max(
			if self.if_four_col(width) {
				let spacing_width = style.spacing.item_spacing.x * 3_f32;
				width
					- self.stored.row_height
					- (self.stored.row_height * 2_f32)
					- spacing_width
			} else {
				width - self.stored.row_height
			} / 2_f32,
		);

		ui.with_layout(Layout::left_to_right(), |ui| {
			if self.if_four_col(width) {
				ui.set_width(self.stored.row_height.max(width_for_cols));
			} else {
				ui.set_max_width(width_for_cols);
			}

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

		let session = find_focused_session(&friend.id, &friend.user_status);

		ui.with_layout(Layout::left_to_right(), |ui| {
			ui.set_width(if self.if_four_col(width) {
				self.stored.row_height.max(width_for_cols)
			} else {
				ui.available_width()
			});
			ui.separator();

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
		});

		if self.if_four_col(width) {
			self.friend_session_thumbnail(ui, frame, session);
		}

		ui.end_row();
	}

	fn user_row(
		&self,
		ui: &mut Ui,
		width: f32,
		frame: &epi::Frame,
		user: &NeosUser,
	) {
		ui.with_layout(Layout::left_to_right(), |ui| {
			let pfp_url: &Option<AssetUrl> = get_user_pfp(user);

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

		let width_for_cols = self
			.stored
			.row_height
			.max((width - self.stored.row_height) / 2_f32);

		// User details
		ui.with_layout(Layout::left_to_right(), |ui| {
			ui.set_max_width(width_for_cols);

			ui.separator();
			ui.vertical(|ui| {
				ui.horizontal(|ui| {
					if user.is_verified {
						ui.label(RichText::new("V").color(Color32::GREEN))
							.on_hover_text("Verified");
					}
					if user.is_locked {
						ui.label(RichText::new("L").color(Color32::RED))
							.on_hover_text("Locked");
					}
					if user.supress_ban_evasion {
						ui.label(RichText::new("B").color(Color32::KHAKI))
							.on_hover_text("Ban evasion disabled");
					}
					ui.add(
						Label::new(RichText::new(&user.username).heading())
							.wrap(true),
					);
				});
				ui.add(
					Label::new(RichText::new(user.id.as_ref()).monospace())
						.wrap(true),
				);
				ui.label(
					RichText::new(
						&user
							.tags
							.iter()
							.filter(|tag| !tag.starts_with("custom badge"))
							.map(std::string::String::as_str)
							.collect::<Vec<&str>>()
							.join(", "),
					)
					.small()
					.monospace(),
				);
			});
		});

		// Bans
		ui.with_layout(Layout::left_to_right(), |ui| {
			ui.set_width(ui.available_width());
			ui.separator();

			ui.vertical(|ui| {
				user_bans(ui, user);
			});
		});

		ui.end_row();
	}

	pub fn peeps_page(&mut self, ui: &mut Ui, frame: &epi::Frame) {
		if self.stored.filter_friends_only {
			self.friends_page(ui, frame);
		} else {
			self.users_page(ui, frame);
		}
	}

	fn users_page(&mut self, ui: &mut Ui, frame: &epi::Frame) {
		use rayon::prelude::*;

		let bar_response = self.search_bar(ui);

		if bar_response.lost_focus() || ui.input().key_pressed(Key::Enter) {
			self.search_users(frame);
		}

		let users: Vec<&NeosUser> = self
			.runtime
			.users
			.par_iter()
			.filter(|user| {
				self.stored.filter_search.is_empty()
					|| user
						.username
						.to_lowercase()
						.contains(&self.stored.filter_search)
					|| user
						.id
						.as_ref()
						.to_lowercase()
						.contains(&self.stored.filter_search)
			})
			.collect();

		let users_count = users.len();

		ui.heading("Peeps search");

		ScrollArea::both().show_rows(
			ui,
			self.stored.row_height,
			users_count,
			|ui, row_range| {
				let width = ui.available_width();
				Grid::new("users_list")
					.start_row(row_range.start)
					.min_row_height(self.stored.row_height)
					.num_columns(3)
					.show(ui, |ui| {
						for row in row_range {
							let user = users[row];
							self.user_row(ui, width, frame, user);
						}
					});
			},
		);
	}

	fn friends_page(&mut self, ui: &mut Ui, frame: &epi::Frame) {
		use rayon::prelude::*;

		self.search_bar(ui);

		let friends: Vec<&NeosFriend> = self
			.runtime
			.friends
			.par_iter()
			.filter(|friend| {
				self.stored.filter_search.is_empty()
					|| friend
						.friend_username
						.to_lowercase()
						.contains(&self.stored.filter_search)
					|| friend
						.id
						.as_ref()
						.to_lowercase()
						.contains(&self.stored.filter_search)
			})
			.collect();

		let friends_count = friends.len();

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
					.num_columns(if self.if_four_col(width) { 4 } else { 3 })
					.show(ui, |ui| {
						for row in row_range {
							let friend = friends[row];
							self.friend_row(ui, width, frame, friend);
						}
					});
			},
		);
	}

	fn friend_session_thumbnail(
		&self,
		ui: &mut Ui,
		frame: &epi::Frame,
		session: Option<&NeosSession>,
	) {
		ui.with_layout(Layout::left_to_right(), |ui| {
			ui.set_min_width(ui.available_width());
			ui.vertical(|ui| {
				if let Some(session) = session {
					if let Some(thumbnail) = &session.thumbnail {
						let session_pics = self.load_texture(thumbnail, frame);

						if let Some(session_pic) = session_pics {
							let scaling = (ui.available_height()
								/ session_pic.size.y)
								.min(ui.available_width() / session_pic.size.x);
							ui.image(
								session_pic.id,
								session_pic.size * scaling,
							);
						}
					}
				}
			});
		});
	}
}

fn user_bans(ui: &mut Ui, user: &NeosUser) {
	ui.label("Ban status");
	let mut any_bans = false;

	if let Some(ban) = &user.public_ban_type {
		any_bans = true;
		ui.label("Ban type: ".to_owned() + ban.as_ref());
	}
	if let Some(acc_ban) = &user.account_ban_expiration {
		any_bans = true;
		ui.label("Account banned until: ".to_owned() + &acc_ban.to_string());
	}
	if let Some(acc_ban) = &user.mute_ban_expiration {
		any_bans = true;
		ui.label("Muted until: ".to_owned() + &acc_ban.to_string());
	}
	if let Some(acc_ban) = &user.public_ban_expiration {
		any_bans = true;
		ui.label("Public ban until: ".to_owned() + &acc_ban.to_string());
	}
	if let Some(acc_ban) = &user.listing_ban_expiration {
		any_bans = true;
		ui.label("Listing ban until: ".to_owned() + &acc_ban.to_string());
	}
	if let Some(acc_ban) = &user.spectator_ban_expiration {
		any_bans = true;
		ui.label("Spectator ban until: ".to_owned() + &acc_ban.to_string());
	}

	if !any_bans {
		ui.label("No bans :)");
	}
}

pub const fn get_friend_pfp(friend: &NeosFriend) -> &Option<AssetUrl> {
	match &friend.profile {
		Some(profile) => &profile.icon_url,
		None => &None,
	}
}
pub const fn get_user_pfp(user: &NeosUser) -> &Option<AssetUrl> {
	match &user.profile {
		Some(profile) => &profile.icon_url,
		None => &None,
	}
}
