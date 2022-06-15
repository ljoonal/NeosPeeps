use eframe::egui::{
	Align,
	Color32,
	Context,
	Grid,
	Id,
	Label,
	Layout,
	RichText,
	ScrollArea,
	Sense,
	Ui,
	Vec2,
	Window,
};

use super::NeosPeepsApp;

pub fn session_users_count(ui: &mut Ui, session: &neos::SessionInfo) {
	ui.horizontal(|ui| {
		ui.style_mut().spacing.item_spacing = Vec2::ZERO;
		ui.label(RichText::new(&session.active_users.to_string()))
			.on_hover_text("Active users");
		ui.label("/");
		ui.label(
			RichText::new(&session.joined_users.to_string()).color(Color32::GRAY),
		)
		.on_hover_text("Joined users");
		ui.label("/");
		ui.label(RichText::new(&session.max_users.to_string()))
			.on_hover_text("Max users");
	});
}

impl NeosPeepsApp {
	pub fn session_window(&mut self, ctx: &Context) {
		let mut open = true;
		if let Some((id, session)) = &*self.runtime.session_window.borrow() {
			Window::new(RichText::new(id.as_ref()).small())
				.id(Id::new("session_window"))
				.open(&mut open)
				.vscroll(true)
				.show(ctx, |ui| {
					if self.threads.loading.session.get() {
						ui.vertical_centered_justified(|ui| {
							ui.label("Loading...");
						});
					} else {
						ui.vertical_centered(|ui| {
							if ui.button("Refresh").clicked() {
								self.get_session(ctx, id);
							}
						});
					}

					if let Some(session) = session {
						if let Some(asset_url) = &session.thumbnail {
							if let Some(thumbnail) = self.load_texture(asset_url, ctx) {
								let size = thumbnail.size_vec2();
								let scaling = (ui.available_height() / size.y)
									.min(ui.available_width() / size.x);
								ui.image(thumbnail.id(), size * scaling);
							}
						}
						ui.horizontal_wrapped(|ui| {
							session_decorations(ui, session);
							ui.add(
								Label::new(RichText::new(session.stripped_name()).heading())
									.wrap(true),
							);
							ui.label(
								RichText::new(session.access_level.as_ref()).small_raised(),
							);
						});

						if !session.tags.is_empty() {
							ui.heading("Tags:");
							ui.horizontal_wrapped(|ui| {
								session_tags(ui, session);
							});
						}

						ui.horizontal_wrapped(|ui| {
							ui.heading("Users: ");
							session_users_count(ui, session);
						});
						if !session.users.is_empty() {
							ui.horizontal_wrapped(|ui| {
								self.session_users(ui, ctx, &session.users);
							});
						}

						ui.horizontal_wrapped(|ui| {
							ui.heading("Host: ");
							if let Some(host_id) = &session.host_id {
								ui.label(host_id.as_ref());
							}
						});

						ui.horizontal_wrapped(|ui| {
							ui.label("Username: ");
							ui.label(&session.host_username);
						});

						ui.horizontal_wrapped(|ui| {
							ui.label("MachineID: ");
							ui.label(&session.host_machine_id);
						});

						ui.heading("Misc");

						ui.horizontal_wrapped(|ui| {
							ui.label("Neos V:");
							ui.label(&session.neos_version);
						});

						ui.horizontal_wrapped(|ui| {
							ui.label("Compatibility hash:");
							ui.label(&session.compatibility_hash);
						});

						ui.horizontal_wrapped(|ui| {
							ui.label("Started at: ");
							ui.label(
								session
									.session_begin_time
									.format(&self.stored.datetime_format)
									.to_string(),
							);
						});

						ui.horizontal_wrapped(|ui| {
							ui.label("Last update at: ");
							ui.label(
								session
									.last_update_time
									.format(&self.stored.datetime_format)
									.to_string(),
							);
						});
					}
				});
		}

		if !open {
			*self.runtime.session_window.borrow_mut() = None;
		}
	}

	pub fn session_row(
		&self, ctx: &Context, ui: &mut Ui, width: f32, session: &neos::SessionInfo,
	) {
		let mut open_window = false;
		ui.with_layout(Layout::top_down(Align::LEFT), |ui| {
			let spacing_width = ui.style().spacing.item_spacing.x;
			ui.set_width(
				self
					.stored
					.col_min_width
					.max(width - (self.stored.row_height * 2_f32) - spacing_width),
			);

			ui.horizontal_wrapped(|ui| {
				if ui
					.add(
						Label::new(
							RichText::new(session.stripped_name())
								.heading()
								.color(Color32::WHITE),
						)
						.wrap(true)
						.sense(Sense::click()),
					)
					.clicked()
				{
					open_window = true;
				}

				session_users_count(ui, session);
			});

			ui.horizontal_wrapped(|ui| {
				ui.label(session.access_level.as_ref());
				ui.label("|");
				if ui
					.add(
						Label::new("Host: ".to_owned() + &session.host_username)
							.wrap(true)
							.sense(Sense::click()),
					)
					.clicked()
				{
					if let Some(user_id) = &session.host_id {
						self.open_user(ctx, user_id, None, None);
					}
				}
			});

			ui.horizontal_wrapped(|ui| {
				ui.label("Users:");
				self.session_users(ui, ctx, &session.users);
			});
			ui.horizontal_wrapped(|ui| {
				ui.label("Tags:");
				session_tags(ui, session);
			});
		});

		ui.with_layout(Layout::top_down(Align::LEFT), |ui| {
			ui.set_min_width(ui.available_width());
			if let Some(asset_url) = &session.thumbnail {
				if let Some(thumbnail) = self.load_texture(asset_url, ctx) {
					let size = thumbnail.size_vec2();
					let scaling =
						(ui.available_height() / size.y).min(ui.available_width() / size.x);
					let response = ui.image(thumbnail.id(), size * scaling);
					if response.interact(Sense::click()).clicked() {
						open_window = true;
					}
				}
			}
		});

		if open_window {
			*self.runtime.session_window.borrow_mut() =
				Some((session.id.clone(), Some(session.clone())));
		}
	}

	pub fn sessions_page(&mut self, ctx: &Context, ui: &mut Ui) {
		use rayon::prelude::*;

		self.search_bar(ui);

		if !self.stored.filter_friends_only && self.threads.loading.sessions.get()
			|| self.stored.filter_friends_only && self.threads.loading.friends.get()
		{
			ui.vertical_centered_justified(|ui| {
				ui.label("Refreshing sessions list");
			});
		}

		let sessions: Vec<&neos::SessionInfo> = if self.stored.filter_friends_only {
			self
				.runtime
				.friends
				.par_iter()
				.flat_map(|friend| &friend.status.active_sessions)
				.collect()
		} else {
			self
				.runtime
				.sessions
				.par_iter()
				.filter(|session| {
					self.stored.filter_search.is_empty()
						|| session
							.host_username
							.to_lowercase()
							.contains(&self.stored.filter_search.to_lowercase())
						|| session
							.stripped_name()
							.to_lowercase()
							.contains(&self.stored.filter_search.to_lowercase())
				})
				.collect()
		};

		ui.heading(sessions.len().to_string() + " Sessions");

		self.sessions_table(
			ctx,
			ui,
			&sessions,
			if self.stored.filter_friends_only {
				"friends_sessions_list"
			} else {
				"sessions_list"
			},
		);
	}

	pub fn sessions_table(
		&self, ctx: &Context, ui: &mut Ui, sessions: &[&neos::SessionInfo],
		id: &str,
	) {
		let sessions_count = sessions.len();

		ScrollArea::vertical().show_rows(
			ui,
			self.stored.row_height,
			sessions_count,
			|ui, row_range| {
				let width = ui.available_width();
				Grid::new(id)
					.striped(true)
					.start_row(row_range.start)
					.min_row_height(self.stored.row_height)
					.num_columns(2)
					.show(ui, |ui| {
						for row in row_range {
							let session = sessions.get(row);
							if let Some(session) = session {
								self.session_row(ctx, ui, width, session);
							} else {
								ui.label("An error occurred");
							}
							ui.end_row();
						}
					});
			},
		);
	}

	fn session_users(
		&self, ui: &mut Ui, ctx: &Context, users: &[neos::SessionUser],
	) {
		use rayon::prelude::*;

		// TODO: Probably possible to do with .par_iter
		for user in users {
			let is_friend = self
				.runtime
				.friends
				.par_iter()
				.find_any(|fren| fren.username == user.username)
				.is_some();

			let text = RichText::new(&user.username).color(
				match (is_friend, user.is_present) {
					(true, true) => Color32::LIGHT_GREEN,
					(true, false) => Color32::GREEN,
					(false, true) => {
						ui.style().visuals.widgets.noninteractive.fg_stroke.color
					}
					(false, false) => Color32::GRAY,
				},
			);

			let label = Label::new(text).sense(Sense::click());

			if ui
				.add(label)
				.on_hover_text(
					match &user.id {
						Some(id) => id.as_ref().to_owned() + " is in ",
						None => "User is in ".to_owned(),
					} + user.output_device.as_ref()
						+ " mode",
				)
				.clicked()
			{
				if let Some(user_id) = &user.id {
					self.open_user(ctx, user_id, None, None);
				}
			}
		}
	}
}

fn session_decorations(ui: &mut Ui, session: &neos::SessionInfo) {
	if !session.is_valid {
		ui.label(RichText::new("!").color(Color32::RED))
			.on_hover_text("Non-valid session");
	}
	if session.has_ended {
		ui.label(RichText::new("E").color(Color32::YELLOW)).on_hover_text("Ended");
	}
	if session.is_mobile_friendly {
		ui.label(RichText::new("M").color(Color32::GREEN))
			.on_hover_text("Mobile friendly session");
	}
}

fn session_tags(ui: &mut Ui, session: &neos::SessionInfo) {
	ui.label(RichText::new(session.tags.join(", ")).small().monospace());
}
