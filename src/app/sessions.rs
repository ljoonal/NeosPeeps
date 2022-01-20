use super::NeosPeepsApp;
use eframe::{
	egui::{
		Align, Color32, CtxRef, Grid, Id, Label, Layout, RichText, ScrollArea,
		Sense, Ui, Vec2, Window,
	},
	epi,
};
use neos::{
	api_client::{AnyNeos, Neos},
	NeosSession, NeosSessionUser, NeosUserStatus,
};

impl NeosPeepsApp {
	/// Refreshes sessions in a background thread
	pub fn refresh_sessions(&mut self, frame: &epi::Frame) {
		use rayon::prelude::*;

		if self.runtime.loading.fetching_sessions
			|| self.runtime.loading.login_op()
		{
			return;
		}
		self.runtime.loading.fetching_sessions = true;
		frame.request_repaint();

		let neos_api_arc = self.runtime.neos_api.clone();
		let sessions_sender = self.channels.sessions_sender();
		rayon::spawn(move || {
			if let AnyNeos::Authenticated(neos_api) = &*neos_api_arc {
				match neos_api.get_sessions() {
					Ok(mut sessions) => {
						sessions.par_sort_by(|s1, s2| {
							s1.active_users.cmp(&s2.active_users).reverse()
						});
						if let Err(err) = sessions_sender.send(sessions) {
							println!(
								"Failed to send sessions to main thread! {}",
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

	/// Gets the session status for the session window
	pub fn get_session(&self, frame: &epi::Frame, id: neos::id::Session) {
		if self.runtime.loading.login_op() {
			return;
		}
		if let Some((w_id, _)) = &*self.runtime.session_window.borrow() {
			if w_id != &id {
				return;
			}
		} else {
			*self.runtime.session_window.borrow_mut() =
				Some((id.clone(), None));
		}

		frame.request_repaint();

		let neos_api = self.runtime.neos_api.clone();
		let session_sender = self.channels.session_sender();
		rayon::spawn(move || match neos_api.get_session(id) {
			Ok(session) => {
				if let Err(err) = session_sender.send(session) {
					println!("Failed to send session to main thread! {}", err);
				}
			}
			Err(e) => {
				println!("Error with Neos API: {}", e);
			}
		});
	}

	pub fn session_window(&mut self, ctx: &CtxRef, frame: &epi::Frame) {
		let mut should_close = false;
		if let Some((id, session)) = &*self.runtime.session_window.borrow() {
			Window::new("Session ".to_owned() + id.as_ref()).show(ctx, |ui| {
				if let Some(session) = session {
					if let Some(asset_url) = &session.thumbnail {
						if let Some(thumbnail) =
							self.load_texture(asset_url, frame)
						{
							let scaling = (ui.available_height()
								/ thumbnail.size.y)
								.min(ui.available_width() / thumbnail.size.x);
							ui.image(thumbnail.id, thumbnail.size * scaling);
						}
					}
				}

				if ui.button("Close").clicked() {
					should_close = true;
				}
			});
		}
		if should_close {
			*self.runtime.session_window.borrow_mut() = None;
		}
	}

	fn session_row(
		&self,
		ui: &mut Ui,
		width: f32,
		frame: &epi::Frame,
		session: &NeosSession,
	) {
		let mut open_window = false;
		ui.with_layout(Layout::top_down(Align::LEFT), |ui| {
			let spacing_width = ui.style().spacing.item_spacing.x;
			ui.set_width(
				self.stored.row_height.max(
					width - (self.stored.row_height * 2_f32) - spacing_width,
				),
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

				ui.horizontal(|ui| {
					ui.style_mut().spacing.item_spacing = Vec2::ZERO;
					ui.label(RichText::new(&session.active_users.to_string()))
						.on_hover_text("Active users");
					ui.label("/");
					ui.label(
						RichText::new(&session.joined_users.to_string())
							.color(Color32::GRAY),
					)
					.on_hover_text("Joined users");
					ui.label("/");
					ui.label(RichText::new(&session.max_users.to_string()))
						.on_hover_text("Max users");
				});
			});

			ui.horizontal_wrapped(|ui| {
				ui.label(session.access_level.as_ref());
				ui.label("|");
				if ui
					.add(
						Label::new(
							"Host: ".to_owned() + &session.host_username,
						)
						.wrap(true)
						.sense(Sense::click()),
					)
					.clicked()
				{
					if let Some(user_id) = &session.host_user_id {
						*self.runtime.user_window.borrow_mut() =
							Some((user_id.clone(), None, None));
						self.get_user(frame, user_id);
					}
				}
			});

			ui.horizontal_wrapped(|ui| {
				self.session_users(ui, frame, &session.session_users);
			});
			ui.horizontal_wrapped(|ui| {
				ui.label("Tags:");
				ui.label(RichText::new(session.tags.join(", ")).small());
			});
		});

		ui.with_layout(Layout::top_down(Align::LEFT), |ui| {
			ui.set_min_width(ui.available_width());
			if let Some(asset_url) = &session.thumbnail {
				if let Some(thumbnail) = self.load_texture(asset_url, frame) {
					let scaling = (ui.available_height() / thumbnail.size.y)
						.min(ui.available_width() / thumbnail.size.x);
					let response =
						ui.image(thumbnail.id, thumbnail.size * scaling);
					if response.interact(Sense::click()).clicked() {
						open_window = true;
					}
				}
			}
		});

		ui.end_row();

		if open_window {
			println!("{}", session.session_id.as_ref());
			*self.runtime.session_window.borrow_mut() =
				Some((session.session_id.clone(), Some(session.clone())));
		}
	}

	pub fn sessions_page(&mut self, ui: &mut Ui, frame: &epi::Frame) {
		use rayon::prelude::*;

		self.search_bar(ui);

		let sessions: Vec<&NeosSession> = if self.stored.filter_friends_only {
			self.runtime
				.friends
				.par_iter()
				.flat_map(|friend| &friend.user_status.active_sessions)
				.collect()
		} else {
			self.runtime
				.sessions
				.par_iter()
				.filter(|session| {
					self.stored.filter_search.is_empty()
						|| session
							.host_username
							.to_lowercase()
							.contains(&self.stored.filter_search)
						|| session
							.stripped_name()
							.to_lowercase()
							.contains(&self.stored.filter_search)
				})
				.collect()
		};

		let sessions_count = sessions.len();

		ui.heading(sessions_count.to_string() + " Sessions");

		ScrollArea::both().show_rows(
			ui,
			self.stored.row_height,
			sessions_count,
			|ui, row_range| {
				let width = ui.available_width();
				Grid::new("sessions_list")
					.striped(true)
					.start_row(row_range.start)
					.min_row_height(self.stored.row_height)
					.num_columns(2)
					.show(ui, |ui| {
						for row in row_range {
							let session = sessions[row];
							self.session_row(ui, width, frame, session);
						}
					});
			},
		);
	}

	fn session_users(
		&self,
		ui: &mut Ui,
		frame: &epi::Frame,
		users: &[NeosSessionUser],
	) {
		use rayon::prelude::*;

		ui.label("Users:");

		// TODO: Probably possible to do with .par_iter
		for user in users {
			let is_friend = self
				.runtime
				.friends
				.par_iter()
				.find_any(|fren| fren.friend_username == user.username)
				.is_some();

			let text = RichText::new(&user.username).color(
				match (is_friend, user.is_present) {
					(true, true) => Color32::LIGHT_GREEN,
					(true, false) => Color32::GREEN,
					(false, true) => {
						ui.style()
							.visuals
							.widgets
							.noninteractive
							.fg_stroke
							.color
					}
					(false, false) => Color32::GRAY,
				},
			);

			let label = Label::new(text).sense(Sense::click());

			if ui
				.add(label)
				.on_hover_text(
					match &user.user_id {
						Some(id) => id.as_ref().to_owned() + " is in ",
						None => "User is in ".to_owned(),
					} + user.output_device.as_ref()
						+ " mode",
				)
				.clicked()
			{
				if let Some(user_id) = &user.user_id {
					*self.runtime.user_window.borrow_mut() =
						Some((user_id.clone(), None, None));
					self.get_user(frame, user_id);
				}
			}
		}
	}
}

pub fn find_focused_session<'a>(
	id: &neos::id::User,
	user_status: &'a NeosUserStatus,
) -> Option<&'a NeosSession> {
	use rayon::prelude::*;

	user_status.active_sessions.par_iter().find_any(|session| {
		session
			.session_users
			.par_iter()
			.find_any(|user| match &user.user_id {
				Some(user_id) => user_id == id && user.is_present,
				None => false,
			})
			.is_some()
	})
}
