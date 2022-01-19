use super::NeosPeepsApp;
use eframe::{
	egui::{Align, Color32, Grid, Label, Layout, RichText, ScrollArea, Ui},
	epi,
};
use neos::{
	api_client::{AnyNeos, Neos},
	NeosSession,
	NeosSessionUser,
	NeosUserStatus,
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

	fn session_row(
		&self,
		ui: &mut Ui,
		width: f32,
		frame: &epi::Frame,
		session: &NeosSession,
	) {
		ui.with_layout(Layout::top_down(Align::LEFT), |ui| {
			let spacing_width = ui.style().spacing.item_spacing.x;
			ui.set_width(
				self.stored.row_height.max(
					width - (self.stored.row_height * 2_f32) - spacing_width,
				),
			);

			ui.horizontal(|ui| {
				ui.add(
					Label::new(
						RichText::new(session.stripped_name()).heading(),
					)
					.wrap(true),
				);
				ui.label(
					RichText::new(&format!(
						"{}/{}/{}",
						&session.active_users,
						&session.joined_users,
						&session.max_users
					))
					.strong(),
				);
			});

			ui.horizontal(|ui| {
				ui.label(session.access_level.as_ref());
				ui.label("|");
				ui.add(
					Label::new("Host: ".to_owned() + &session.host_username)
						.wrap(true),
				);
			});

			ui.horizontal_wrapped(|ui| {
				self.session_users(ui, &session.session_users);
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
					ui.image(thumbnail.id, thumbnail.size * scaling);
				}
			}
		});

		ui.end_row();
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

	fn session_users(&self, ui: &mut Ui, users: &[NeosSessionUser]) {
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

			ui.label(text).on_hover_text(
				match &user.user_id {
					Some(id) => id.as_ref().to_owned() + " is in ",
					None => "User is in ".to_owned(),
				} + user.output_device.as_ref()
					+ " mode",
			);
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
