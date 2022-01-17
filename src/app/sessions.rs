use super::NeosPeepsApp;
use eframe::{
	egui::{Grid, ScrollArea, Ui},
	epi,
};
use neos::{
	api_client::{AnyNeos, Neos},
	NeosSession,
	NeosUserStatus,
};

impl NeosPeepsApp {
	/// Refreshes sessions in a background thread
	pub fn refresh_sessions(&mut self, frame: epi::Frame) {
		{
			let mut loading = self.runtime.loading.write().unwrap();
			if loading.is_loading() {
				return;
			}
			*loading = crate::data::LoadingState::FetchingSessions;
		}
		frame.request_repaint();

		let neos_api_arc = self.runtime.neos_api.clone();
		let sessions_arc = self.runtime.sessions.clone();
		let loading = self.runtime.loading.clone();
		rayon::spawn(move || {
			if let AnyNeos::Authenticated(neos_api) =
				&*neos_api_arc.read().unwrap()
			{
				match neos_api.get_sessions() {
					Ok(sessions) => {
						*sessions_arc.write().unwrap() = sessions;
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

	pub fn sessions_page(&mut self, ui: &mut Ui, frame: &epi::Frame) {
		ui.heading("Coming soon!");

		let sessions = self.runtime.sessions.read().unwrap();

		ScrollArea::both().show_rows(
			ui,
			self.stored.row_height,
			sessions.len(),
			|ui, row_range| {
				ui.set_width(ui.available_width());
				Grid::new("sessions_list")
					.start_row(row_range.start)
					.min_col_width(self.stored.row_height)
					//.num_columns(3)
					.show(ui, |ui| {
						ui.set_height(self.stored.row_height);
						ui.set_width(ui.available_width());
						for row in row_range {
							let session = &sessions[row];
							if !session.has_ended && session.is_valid {
								ui.label(&session.name);
							}
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

fn has_session_thumbnail(user_status: &NeosUserStatus, asset_id: &str) -> bool {
	use rayon::prelude::*;

	user_status
		.active_sessions
		.par_iter()
		.find_any(|session| match &session.thumbnail {
			Some(thumbnail) => thumbnail.id() == asset_id,
			None => false,
		})
		.is_some()
}
