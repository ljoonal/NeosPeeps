//! The friends page of the app
use std::time::SystemTime;

use eframe::egui::{
	Align,
	Color32,
	Context,
	Grid,
	Id,
	Key,
	Label,
	Layout,
	RichText,
	ScrollArea,
	Sense,
	Ui,
	Vec2,
	Window,
};

use super::{sessions::session_users_count, NeosPeepsApp};
use crate::sessions::find_focused_session;

impl NeosPeepsApp {
	pub fn user_window(&mut self, ctx: &Context) {
		let mut open = true;
		if let Some((id, user, status)) = &*self.runtime.user_window.borrow() {
			Window::new(id.as_ref())
				.open(&mut open)
				.id(Id::new("user_window"))
				.vscroll(true)
				.show(ctx, |ui| {
					if self.threads.loading.user.get() {
						ui.vertical_centered_justified(|ui| {
							ui.label("Loading user...");
						});
					} else {
						ui.vertical_centered(|ui| {
							if ui.button("Refresh user").clicked() {
								self.get_user(ctx, id);
							}
						});
					}

					if let Some(user) = user {
						self.user_window_section_user(ui, ctx, user);
					}

					ui.separator();

					if self.threads.loading.user_status.get() {
						ui.vertical_centered_justified(|ui| {
							ui.label("Loading user status...");
						});
					} else {
						ui.vertical_centered(|ui| {
							if ui.button("Refresh status").clicked() {
								self.get_user_status(ctx, id);
							}
						});
					}

					if let Some(status) = status {
						self.user_window_section_status(ctx, ui, status);
					}
				});
		}
		if !open {
			*self.runtime.user_window.borrow_mut() = None;
		}
	}

	fn user_window_section_user(
		&self, ui: &mut Ui, ctx: &Context, user: &neos::User,
	) {
		let pfp = self.get_pfp(ctx, &user.profile);
		let size = pfp.size_vec2();
		let scaling =
			(ui.available_height() / size.y).min(ui.available_width() / size.x);
		ui.image(pfp.id(), size * scaling);

		let friend = self.user_to_friend(user);

		ui.horizontal_wrapped(|ui| {
			username_decorations(ui, user, friend);
			ui.heading(&user.username);
			if friend.is_some() {
				if ui.button("Remove").on_hover_text("Remove from contacts").clicked() {
					self.remove_friend(user.id.clone());
				}
				if ui.button("Chat").on_hover_text("Read/Send messages").clicked() {
					*self.runtime.open_chat.borrow_mut() =
						Some((user.id.clone(), String::new(), SystemTime::UNIX_EPOCH));
				}
			} else if ui
				.button("Add")
				.on_hover_text("Add to contacts (=send friend request)")
				.clicked()
			{
				self.add_friend(user.id.clone());
			}
		});

		if let Some(friend) = friend {
			if let Some(msg_time) = &friend.latest_message_time {
				ui.horizontal_wrapped(|ui| {
					ui.label("Last message time: ");
					ui.label(self.runtime.format_time(msg_time));
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
			for (token, amount) in credits {
				ui.horizontal_wrapped(|ui| {
					ui.label(token.clone() + ": ");
					ui.label(amount.to_string());
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
		&self, ctx: &Context, ui: &mut Ui, status: &neos::UserStatus,
	) {
		let (r, g, b) = status.online_status.color();
		ui.label(
			RichText::new(&status.online_status.to_string())
				.color(Color32::from_rgb(r, g, b)),
		);

		if let Some(status_change) = status.last_status_change_time {
			ui.horizontal_wrapped(|ui| {
				ui.label("Status last changed on:");
				ui.label(self.runtime.format_time(&status_change));
			});
		}

		if let Some(neos_version) = &status.neos_version {
			ui.horizontal_wrapped(|ui| {
				ui.label("Neos V:");
				ui.label(neos_version);
			});
		}

		if let Some(hash) = &status.compatibility_hash {
			ui.horizontal_wrapped(|ui| {
				ui.label("Compatibility hash:");
				ui.label(hash);
			});
		}

		ui.horizontal_wrapped(|ui| {
			ui.label("Output device:");
			ui.label(status.output_device.as_ref());
		});

		ui.horizontal_wrapped(|ui| {
			ui.label("Mobile:");
			ui.label(status.is_mobile.to_string());
		});

		ui.horizontal_wrapped(|ui| {
			ui.label("Current session is hidden:");
			ui.label(status.is_current_session_hidden.to_string());
		});

		ui.horizontal_wrapped(|ui| {
			ui.label("Hosting:");
			ui.label(status.is_current_hosting.to_string());
		});

		if !status.active_sessions.is_empty() {
			ui.collapsing("Sessions", |ui| {
				for session in &status.active_sessions {
					self.session_row(ctx, ui, ui.available_width(), session);
					ui.end_row();
				}
			});
		}
	}

	fn friend_row(
		&self, ctx: &Context, ui: &mut Ui, width: f32, friend: &neos::Friend,
	) {
		ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
			let pfp = self.get_pfp(ctx, &friend.profile);

			let response = ui.image(
				pfp.id(),
				Vec2::new(self.stored.row_height, self.stored.row_height),
			);

			if response.interact(Sense::click()).clicked() {
				self.open_user(ctx, &friend.id, None, None);
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

		ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
			ui.set_width(width_for_cols);

			ui.separator();
			ui.vertical(|ui| {
				let (r, g, b) = friend.status.online_status.color();
				self.clickable_username(
					ui,
					ctx,
					&friend.id,
					&friend.username,
					None,
					None,
				);
				self.clickable_user_id(ui, ctx, &friend.id, None, None);
				ui.label(
					RichText::new(&friend.status.online_status.to_string())
						.color(Color32::from_rgb(r, g, b)),
				);

				if self.stored.row_height >= 130f32 {
					let response = if let Some(time) = friend.latest_message_time {
						ui.add(
							Label::new(self.runtime.format_time(&time)).sense(Sense::click()),
						)
					} else {
						ui.add(Label::new("No messages").sense(Sense::click()))
					};

					if response.clicked() {
						*self.runtime.open_chat.borrow_mut() =
							Some((friend.id.clone(), String::new(), SystemTime::UNIX_EPOCH));
					}
				}
			});
		});

		ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
			ui.set_width(ui.available_width());

			ui.separator();

			self.friend_row_session_col(ctx, ui, width_for_cols, friend);
		});
	}

	fn friend_row_session_col(
		&self, ctx: &Context, ui: &mut Ui, width: f32, friend: &neos::Friend,
	) {
		if let Some(session) = find_focused_session(&friend.id, &friend.status) {
			let show_thumbnail = width > self.stored.row_height;
			ui.vertical(|ui| {
				if show_thumbnail {
					ui.set_width(width);
				}
				if ui
					.add(
						Label::new(session.stripped_name())
							.wrap(true)
							.sense(Sense::click()),
					)
					.clicked()
				{
					*self.runtime.session_window.borrow_mut() =
						Some((session.id.clone(), Some(session.clone())));
				}
				ui.label(friend.status.current_session_access_level.as_ref());
				session_users_count(ui, session);
			});
			if show_thumbnail {
				self.friend_session_thumbnail(ctx, ui, session);
			}
		} else if friend.status.online_status == neos::OnlineStatus::Offline {
			ui.label(friend.status.online_status.as_ref());
		} else {
			ui.vertical(|ui| {
				ui.label("Couldn't find focused session");
				ui.label(friend.status.current_session_access_level.as_ref());
			});
		}
	}

	fn user_row(&self, ctx: &Context, ui: &mut Ui, user: &neos::User) {
		ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
			let pfp = self.get_pfp(ctx, &user.profile);

			let response = ui.image(
				pfp.id(),
				Vec2::new(self.stored.row_height, self.stored.row_height),
			);

			if response.interact(Sense::click()).clicked() {
				self.open_user(ctx, &user.id, Some(user.clone()), None);
			}
		});

		// User details
		ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
			ui.set_width(ui.available_width());

			ui.separator();
			ui.vertical(|ui| {
				ui.horizontal(|ui| {
					username_decorations(ui, user, self.user_to_friend(user));
					self.clickable_username(
						ui,
						ctx,
						&user.id,
						&user.username,
						Some(user),
						None,
					);
				});

				self.clickable_user_id(ui, ctx, &user.id, Some(user), None);
				user_tags(ui, user);
				user_bans(ui, user);
			});
		});
	}

	pub fn peeps_page(&mut self, ctx: &Context, ui: &mut Ui) {
		if self.runtime.open_chat.borrow().is_some() {
			self.chat_page(ctx, ui);
		} else if self.stored.filter_friends_only {
			self.friends_page(ctx, ui);
		} else {
			self.users_page(ctx, ui);
		}
	}

	fn users_page(&mut self, ctx: &Context, ui: &mut Ui) {
		use rayon::prelude::*;

		let bar_response = self.search_bar(ui);

		if bar_response.lost_focus() || ui.input().key_pressed(Key::Enter) {
			self.search_users(ctx);
		}

		if self.threads.loading.users.get() {
			ui.vertical_centered_justified(|ui| {
				ui.label("Searching...");
			});
		}

		let users: Vec<&neos::User> = self
			.runtime
			.users
			.par_iter()
			.filter(|user| {
				self.stored.filter_search.is_empty()
					|| user
						.username
						.to_lowercase()
						.contains(&self.stored.filter_search.to_lowercase())
					|| user
						.id
						.as_ref()
						.to_lowercase()
						.contains(&self.stored.filter_search.to_lowercase())
			})
			.collect();

		let users_count = users.len();

		ui.heading("Peeps search");

		ScrollArea::both().show_rows(
			ui,
			self.stored.row_height,
			users_count,
			|ui, row_range| {
				Grid::new("users_list")
					.start_row(row_range.start)
					.striped(true)
					.min_row_height(self.stored.row_height)
					.num_columns(2)
					.show(ui, |ui| {
						for row in row_range {
							let user = users.get(row);
							if let Some(user) = user {
								self.user_row(ctx, ui, user);
							} else {
								ui.label("An error occurred");
							}
						}
					});
			},
		);
	}

	fn friends_page(&mut self, ctx: &Context, ui: &mut Ui) {
		use rayon::prelude::*;

		self.search_bar(ui);

		if self.threads.loading.friends.get() {
			ui.vertical_centered_justified(|ui| {
				ui.label("Refreshing friends list");
			});
		} else if self.threads.loading.messages.get() {
			ui.vertical_centered_justified(|ui| {
				ui.label("Refreshing messages");
			});
		}

		let friends: Vec<&neos::Friend> = self
			.runtime
			.friends
			.par_iter()
			.filter(|friend| {
				self.stored.filter_search.is_empty()
					|| friend
						.username
						.to_lowercase()
						.contains(&self.stored.filter_search.to_lowercase())
					|| friend
						.id
						.as_ref()
						.to_lowercase()
						.contains(&self.stored.filter_search.to_lowercase())
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
							let friend = friends.get(row);
							if let Some(friend) = friend {
								self.friend_row(ctx, ui, width, friend);
							} else {
								ui.label("An error occurred");
							}
							ui.end_row();
						}
					});
			},
		);
	}

	pub fn clickable_username(
		&self, ui: &mut Ui, ctx: &Context, id: &neos::id::User, username: &str,
		user: Option<&neos::User>, user_status: Option<&neos::UserStatus>,
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
				ctx,
				id,
				user.map(Clone::clone),
				user_status.map(Clone::clone),
			);
		}
	}

	pub fn clickable_user_id(
		&self, ui: &mut Ui, ctx: &Context, id: &neos::id::User,
		user: Option<&neos::User>, user_status: Option<&neos::UserStatus>,
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
				ctx,
				id,
				user.map(Clone::clone),
				user_status.map(Clone::clone),
			);
		}
	}

	fn friend_session_thumbnail(
		&self, ctx: &Context, ui: &mut Ui, session: &neos::SessionInfo,
	) {
		if let Some(thumbnail) = &session.thumbnail {
			ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
				ui.set_width(ui.available_width());
				let session_pics = self.load_texture(thumbnail, ctx);
				if let Some(session_pic) = session_pics {
					let size = session_pic.size_vec2();
					let scaling =
						(ui.available_height() / size.y).min(ui.available_width() / size.x);
					let response = ui.image(session_pic.id(), size * scaling);

					if response.interact(Sense::click()).clicked() {
						*self.runtime.session_window.borrow_mut() =
							Some((session.id.clone(), Some(session.clone())));
					}
				}
			});
		}
	}
}

fn username_decorations(
	ui: &mut Ui, user: &neos::User, friend: Option<&neos::Friend>,
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

fn user_tags(ui: &mut Ui, user: &neos::User) {
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

fn user_bans(ui: &mut Ui, user: &neos::User) {
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
