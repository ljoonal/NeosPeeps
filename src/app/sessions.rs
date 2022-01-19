use super::NeosPeepsApp;
use eframe::{
	egui::{self, Grid, Label, Layout, RichText, ScrollArea, Ui, Vec2},
	epi,
};
use neos::{
	api_client::{AnyNeos, Neos},
	NeosSession, NeosUserStatus,
};

impl NeosPeepsApp {
	/// Refreshes sessions in a background thread
	pub fn refresh_sessions(&mut self, frame: epi::Frame) {
		{
			let mut loading = self.runtime.loading.write().unwrap();
			if loading.fetching_sessions || loading.login_op() {
				return;
			}
			loading.fetching_sessions = true;
		}
		frame.request_repaint();

		let neos_api_arc = self.runtime.neos_api.clone();
		let sessions_arc = self.runtime.sessions.clone();
		let loading = self.runtime.loading.clone();
		rayon::spawn(move || {
			if let AnyNeos::Authenticated(neos_api) = &*neos_api_arc {
				match neos_api.get_sessions() {
					Ok(sessions) => {
						*sessions_arc.write().unwrap() = sessions;
					}
					Err(e) => {
						println!("Error with Neos API: {}", e);
					}
				}
			}

			loading.write().unwrap().fetching_sessions = false;
			frame.request_repaint();
		});
	}

	fn session_row(
		&self,
		ui: &mut Ui,
		width: f32,
		frame: &epi::Frame,
		session: &NeosSession,
	) {
		ui.with_layout(Layout::left_to_right(), |ui| {
			egui::trace!(ui);
			ui.set_max_width(width - (self.stored.row_height * 2.1));
			ui.vertical(|ui| {
				egui::trace!(ui);
				ui.heading(session.stripped_name());
				ui.label("Host: ".to_owned() + &session.host_username);

				ui.horizontal(|ui| {
					ui.label(session.access_level.as_ref());
					ui.label(&format!(
						"{}/{}",
						&session.joined_users, &session.max_users
					));
				});

				ui.label(RichText::new(session.tags.join(", ")).small());
			});
		});

		ui.with_layout(Layout::left_to_right(), |ui| {
			ui.set_min_width(ui.available_width());
			if let Some(asset_url) = &session.thumbnail {
				if let Some(thumbnail) =
					self.runtime.load_texture(asset_url, frame)
				{
					let scaling = ui.available_height() / thumbnail.size.y;
					ui.image(thumbnail.id, thumbnail.size * scaling);
				}
			}
		});

		ui.end_row();
	}

	pub fn sessions_page(&mut self, ui: &mut Ui, frame: &epi::Frame) {
		let sessions = self.runtime.sessions.read().unwrap();
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
							let session = &sessions[row];
							self.session_row(ui, width, frame, session);
						}
					});
			},
		);
	}
}

/*
pub fn load_all_user_session_thumbnails(
	sessions: &[NeosSession],
	pics: &Arc<RwLock<TexturesMap>>,
	frame: &epi::Frame,
) {
	use rayon::prelude::*;

	sessions.par_iter().for_each(|session| {
		if let Some(url) = &session.thumbnail {
			self.load_texture(url, pics.clone(), frame);
		}
	});
}
*/

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
