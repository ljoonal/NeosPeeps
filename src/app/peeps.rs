//! The friends page of the app

use std::{cmp::Ordering, rc::Rc};

use eframe::{
	egui::{
		Color32,
		CtxRef,
		Grid,
		Key,
		Label,
		Layout,
		RichText,
		ScrollArea,
		Sense,
		Ui,
		Vec2,
		Window,
	},
	epi,
};
use neos::{
	api_client::{AnyNeos, Neos},
	NeosFriend,
	NeosSession,
	NeosUser,
	NeosUserOnlineStatus,
	NeosUserProfile,
	NeosUserStatus,
};

use super::{
	sessions::{find_focused_session, session_users_count},
	NeosPeepsApp,
};
use crate::image::TextureDetails;

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
						friends.sort_by(|f1, f2| order_users(&f1.status, &f2.status));
						if let Err(err) = friends_sender.send(friends) {
							println!("Failed to send friends to main thread! {}", err);
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
		if self.stored.filter_search.is_empty() || self.runtime.loading.login_op() {
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
	pub fn get_user(&self, frame: &epi::Frame, id: &neos::id::User) {
		if self.runtime.loading.login_op() {
			return;
		}
		if let Some((w_id, _, _)) = &*self.runtime.user_window.borrow() {
			if w_id != id {
				return;
			}
		} else {
			*self.runtime.user_window.borrow_mut() = Some((id.clone(), None, None));
		}

		frame.request_repaint();

		let id = id.clone();
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
	pub fn get_user_status(&self, frame: &epi::Frame, id: &neos::id::User) {
		if self.runtime.loading.login_op() {
			return;
		}
		if let Some((w_id, _, _)) = &*self.runtime.user_window.borrow() {
			if w_id != id {
				return;
			}
		} else {
			*self.runtime.user_window.borrow_mut() = Some((id.clone(), None, None));
		}

		frame.request_repaint();

		let id = id.clone();
		let neos_api = self.runtime.neos_api.clone();
		let user_status_sender = self.channels.user_status_sender();
		rayon::spawn(move || match neos_api.get_user_status(id.clone()) {
			Ok(user_status) => {
				if let Err(err) = user_status_sender.send((id, user_status)) {
					println!("Failed to send user status to main thread! {}", err);
				}
			}
			Err(e) => {
				println!("Error with Neos API: {}", e);
			}
		});
	}

	pub fn user_window(&mut self, ctx: &CtxRef, frame: &epi::Frame) {
		let mut open = true;
		let mut refresh_user: Option<neos::id::User> = None;
		let mut refresh_user_status: Option<neos::id::User> = None;
		if let Some((id, user, status)) = &*self.runtime.user_window.borrow() {
			Window::new(id.as_ref()).open(&mut open).vscroll(true).show(ctx, |ui| {
				if let Some(user) = user {
					ui.vertical_centered(|ui| {
						if ui.button("Refresh user").clicked() {
							refresh_user = Some(id.clone());
						}
					});
					self.user_window_section_user(ui, frame, user);
				}

				if let Some(status) = status {
					ui.separator();
					ui.vertical_centered(|ui| {
						if ui.button("Refresh status").clicked() {
							refresh_user_status = Some(id.clone());
						}
					});

					self.user_window_section_status(ui, frame, status);
				}
			});
		}
		if !open {
			*self.runtime.user_window.borrow_mut() = None;
		} else if let Some(id) = refresh_user {
			if let Some(w_user) = &mut *self.runtime.user_window.borrow_mut() {
				w_user.1 = None;
			}
			self.get_user(frame, &id);
		} else if let Some(id) = refresh_user_status {
			if let Some(w_user) = &mut *self.runtime.user_window.borrow_mut() {
				w_user.2 = None;
			}
			self.get_user_status(frame, &id);
		}
	}

	fn user_window_section_user(
		&self, ui: &mut Ui, frame: &epi::Frame, user: &NeosUser,
	) {
		let pfp = self.get_pfp(frame, &user.profile);
		let scaling = (ui.available_height() / pfp.size.y)
			.min(ui.available_width() / pfp.size.x);
		ui.image(pfp.id, pfp.size * scaling);

		let friend = self.user_to_friend(user);

		ui.horizontal_wrapped(|ui| {
			username_decorations(ui, user, friend);
			ui.heading(&user.username);
		});

		if let Some(friend) = friend {
			if let Some(msg_time) = &friend.latest_message_time {
				ui.horizontal_wrapped(|ui| {
					ui.label("Last message time: ");
					ui.label(msg_time.format(&self.stored.datetime_format).to_string());
				});
			}
		}

		#[allow(clippy::cast_precision_loss)]
		if user.used_bytes.is_some() || user.quota_bytes.is_some() {
			ui.horizontal_wrapped(|ui| {
				ui.label("GB used: ");
				ui.label(user.used_bytes.map_or_else(
					|| "?".to_string(),
					|v| format!("{:.3}", v as f64 / 1_000_000_000_f64),
				));
				ui.label("/");
				ui.label(user.quota_bytes.map_or_else(
					|| "?".to_string(),
					|v| format!("{:.3}", v as f64 / 1_000_000_000_f64),
				));
			});
		}

		if let Some(email) = &user.email {
			ui.horizontal_wrapped(|ui| {
				ui.label("Email: ");
				ui.label(email);
			});
		}

		if user.two_factor_login {
			ui.label("Two factor enabled");
		}

		if !user.tags.is_empty() {
			ui.horizontal_wrapped(|ui| {
				ui.label("Tags: ");
				user_tags(ui, user);
			});
		}

		if let Some(referral_id) = &user.referral_id {
			ui.horizontal_wrapped(|ui| {
				ui.label("Referral ID: ");
				ui.label(referral_id);
			});
		}

		if let Some(credits) = &user.credits {
			if let Some(ncr) = credits.ncr {
				ui.horizontal_wrapped(|ui| {
					ui.label("NCR: ");
					ui.label(ncr.to_string());
				});
			}
			if let Some(kfc) = credits.kfc {
				ui.horizontal_wrapped(|ui| {
					ui.label("KFC: ");
					ui.label(kfc.to_string());
				});
			}
		}

		if let Some(addr) = &user.ncr_deposit_address {
			ui.horizontal_wrapped(|ui| {
				ui.label("NCR deposit addr: ");
				ui.label(addr);
			});
		}

		user_bans(ui, user);
	}

	fn user_window_section_status(
		&self, ui: &mut Ui, frame: &epi::Frame, status: &NeosUserStatus,
	) {
		let (r, g, b) = status.online_status.color();
		ui.label(
			RichText::new(&status.online_status.to_string())
				.color(Color32::from_rgb(r, g, b)),
		);

		if let Some(status_change) = status.last_status_change_time {
			ui.horizontal(|ui| {
				ui.label("Status last changed on:");
				ui.label(
					status_change.format(&self.stored.datetime_format).to_string(),
				);
			});
		}

		if let Some(neos_version) = &status.neos_version {
			ui.horizontal(|ui| {
				ui.label("Neos V:");
				ui.label(neos_version);
			});
		}

		if let Some(hash) = &status.compatibility_hash {
			ui.horizontal(|ui| {
				ui.label("Compatibility hash:");
				ui.label(hash);
			});
		}

		ui.horizontal(|ui| {
			ui.label("Output device:");
			ui.label(status.output_device.as_ref());
		});

		ui.horizontal(|ui| {
			ui.label("Mobile:");
			ui.label(status.is_mobile.to_string());
		});

		ui.horizontal(|ui| {
			ui.label("Current session is hidden:");
			ui.label(status.is_current_session_hidden.to_string());
		});

		ui.horizontal(|ui| {
			ui.label("Hosting:");
			ui.label(status.is_current_hosting.to_string());
		});

		if !status.active_sessions.is_empty() {
			ui.group(|ui| {
				self.sessions_table(
					ui,
					frame,
					status
						.active_sessions
						.iter()
						.collect::<Vec<&NeosSession>>()
						.as_slice(),
				);
			});
		}
	}

	fn friend_row(
		&self, ui: &mut Ui, width: f32, frame: &epi::Frame, friend: &NeosFriend,
	) {
		ui.with_layout(Layout::left_to_right(), |ui| {
			let pfp = self.get_pfp(frame, &friend.profile);

			let response = ui.image(
				pfp.id,
				Vec2::new(self.stored.row_height, self.stored.row_height),
			);

			if response.interact(Sense::click()).clicked() {
				self.open_user(frame, &friend.id, None, None);
			}
		});
		// The width for 2 each of the "columns" (last one not really) before
		// the thumbnail.
		let width_for_cols = self.stored.col_min_width.max(
			(width
				- self.stored.row_height
				- (self.stored.col_min_width * 2_f32)
				- (ui.style().spacing.item_spacing.x * 3_f32))
				/ 2_f32,
		);

		ui.with_layout(Layout::left_to_right(), |ui| {
			ui.set_width(width_for_cols);

			ui.separator();
			ui.vertical(|ui| {
				let (r, g, b) = friend.status.online_status.color();
				self.clickable_username(
					ui,
					frame,
					&friend.id,
					&friend.username,
					None,
					None,
				);
				ui.label(
					RichText::new(&friend.status.online_status.to_string())
						.color(Color32::from_rgb(r, g, b)),
				);
				self.clickable_user_id(ui, frame, &friend.id, None, None);
			});
		});

		ui.with_layout(Layout::left_to_right(), |ui| {
			ui.set_width(ui.available_width());

			ui.separator();

			self.friend_row_session_col(ui, width_for_cols, frame, friend);
		});

		ui.end_row();
	}

	fn friend_row_session_col(
		&self, ui: &mut Ui, width: f32, frame: &epi::Frame, friend: &NeosFriend,
	) {
		if let Some(session) = find_focused_session(&friend.id, &friend.status) {
			let show_thumbnail = width > self.stored.row_height;
			ui.vertical(|ui| {
				if show_thumbnail {
					ui.set_width(width);
				}
				if ui
					.add(Label::new(&session.name).wrap(true).sense(Sense::click()))
					.clicked()
				{
					*self.runtime.session_window.borrow_mut() =
						Some((session.id.clone(), Some(session.clone())));
				}
				ui.label(friend.status.current_session_access_level.as_ref());
				session_users_count(ui, session);
			});
			if show_thumbnail {
				self.friend_session_thumbnail(ui, frame, session);
			}
		} else if friend.status.online_status == NeosUserOnlineStatus::Offline {
			ui.label(friend.status.online_status.as_ref());
		} else {
			ui.vertical(|ui| {
				ui.label("Couldn't find focused session");
				ui.label(friend.status.current_session_access_level.as_ref());
			});
		}
	}

	fn user_row(
		&self, ui: &mut Ui, width: f32, frame: &epi::Frame, user: &NeosUser,
	) {
		ui.with_layout(Layout::left_to_right(), |ui| {
			let pfp = self.get_pfp(frame, &user.profile);

			let response = ui.image(
				pfp.id,
				Vec2::new(self.stored.row_height, self.stored.row_height),
			);

			if response.interact(Sense::click()).clicked() {
				self.open_user(frame, &user.id, Some(user.clone()), None);
			}
		});

		// User details
		ui.with_layout(Layout::left_to_right(), |ui| {
			ui.set_width(ui.available_width());

			ui.separator();
			ui.vertical(|ui| {
				ui.horizontal(|ui| {
					username_decorations(ui, user, self.user_to_friend(user));
					self.clickable_username(
						ui,
						frame,
						&user.id,
						&user.username,
						Some(user),
						None,
					);
				});

				self.clickable_user_id(ui, frame, &user.id, Some(user), None);
				user_tags(ui, user);
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
					|| user.username.to_lowercase().contains(&self.stored.filter_search)
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
					.striped(true)
					.min_row_height(self.stored.row_height)
					.num_columns(2)
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
					|| friend.username.to_lowercase().contains(&self.stored.filter_search)
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
					.striped(true)
					.min_row_height(self.stored.row_height)
					.num_columns(3)
					.show(ui, |ui| {
						for row in row_range {
							let friend = friends[row];
							self.friend_row(ui, width, frame, friend);
						}
					});
			},
		);
	}

	fn clickable_username(
		&self, ui: &mut Ui, frame: &epi::Frame, id: &neos::id::User,
		username: &str, user: Option<&NeosUser>,
		user_status: Option<&NeosUserStatus>,
	) {
		if ui
			.add(
				Label::new(RichText::new(username).heading().color(Color32::WHITE))
					.wrap(true)
					.sense(Sense::click()),
			)
			.clicked()
		{
			self.open_user(
				frame,
				id,
				user.map(Clone::clone),
				user_status.map(Clone::clone),
			);
		}
	}

	fn clickable_user_id(
		&self, ui: &mut Ui, frame: &epi::Frame, id: &neos::id::User,
		user: Option<&NeosUser>, user_status: Option<&NeosUserStatus>,
	) {
		if ui
			.add(
				Label::new(RichText::new(id.as_ref()).monospace())
					.wrap(true)
					.sense(Sense::click()),
			)
			.clicked()
		{
			self.open_user(
				frame,
				id,
				user.map(Clone::clone),
				user_status.map(Clone::clone),
			);
		}
	}

	pub fn open_user(
		&self, frame: &epi::Frame, id: &neos::id::User, user: Option<NeosUser>,
		user_status: Option<NeosUserStatus>,
	) {
		let (missing_user, missing_status) =
			(user.is_none(), user_status.is_none());
		*self.runtime.user_window.borrow_mut() =
			Some((id.clone(), user, user_status));
		if missing_user {
			self.get_user(frame, id);
		}
		if missing_status {
			self.get_user_status(frame, id);
		}
	}

	fn friend_session_thumbnail(
		&self, ui: &mut Ui, frame: &epi::Frame, session: &NeosSession,
	) {
		if let Some(thumbnail) = &session.thumbnail {
			ui.with_layout(Layout::right_to_left(), |ui| {
				ui.set_width(ui.available_width());
				let session_pics = self.load_texture(thumbnail, frame);
				if let Some(session_pic) = session_pics {
					let scaling = (ui.available_height() / session_pic.size.y)
						.min(ui.available_width() / session_pic.size.x);
					let response = ui.image(session_pic.id, session_pic.size * scaling);

					if response.interact(Sense::click()).clicked() {
						*self.runtime.session_window.borrow_mut() =
							Some((session.id.clone(), Some(session.clone())));
					}
				}
			});
		}
	}

	fn get_pfp(
		&self, frame: &epi::Frame, profile: &Option<NeosUserProfile>,
	) -> Rc<TextureDetails> {
		let pfp_url = match profile {
			Some(profile) => &profile.icon_url,
			None => &None,
		};
		let pfp = match pfp_url {
			Some(pfp_url) => self.load_texture(pfp_url, frame),
			None => None,
		};

		pfp.unwrap_or_else(|| self.runtime.default_profile_picture.clone().unwrap())
	}

	fn user_to_friend(&self, user: &NeosUser) -> Option<&NeosFriend> {
		use rayon::prelude::*;

		self.runtime.friends.par_iter().find_any(|f| f.id == user.id)
	}
}

fn username_decorations(
	ui: &mut Ui, user: &NeosUser, friend: Option<&NeosFriend>,
) {
	if let Some(as_friend) = friend {
		if as_friend.is_accepted {
			ui.label(RichText::new("F").color(Color32::from_rgb(255, 0, 122)))
				.on_hover_text("Friend");
		} else {
			ui.label(RichText::new("R").color(Color32::YELLOW))
				.on_hover_text("Requested friendship");
		}
	}

	if user.is_verified {
		ui.label(RichText::new("V").color(Color32::GREEN))
			.on_hover_text("Verified (email)");
	}
	if user.is_locked {
		ui.label(RichText::new("L").color(Color32::RED)).on_hover_text("Locked");
	}
	if user.supress_ban_evasion {
		ui.label(RichText::new("B").color(Color32::KHAKI))
			.on_hover_text("Ban evasion disabled");
	}
}

fn user_tags(ui: &mut Ui, user: &NeosUser) {
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
}

fn user_bans(ui: &mut Ui, user: &NeosUser) {
	if let Some(ban) = &user.public_ban_type {
		ui.label("Ban type: ".to_owned() + ban.as_ref());
	}
	if let Some(acc_ban) = &user.account_ban_expiration {
		ui.label("Account banned until: ".to_owned() + &acc_ban.to_string());
	}
	if let Some(acc_ban) = &user.mute_ban_expiration {
		ui.label("Muted until: ".to_owned() + &acc_ban.to_string());
	}
	if let Some(acc_ban) = &user.public_ban_expiration {
		ui.label("Public ban until: ".to_owned() + &acc_ban.to_string());
	}
	if let Some(acc_ban) = &user.listing_ban_expiration {
		ui.label("Listing ban until: ".to_owned() + &acc_ban.to_string());
	}
	if let Some(acc_ban) = &user.spectator_ban_expiration {
		ui.label("Spectator ban until: ".to_owned() + &acc_ban.to_string());
	}
}
