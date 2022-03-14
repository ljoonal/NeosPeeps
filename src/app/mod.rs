use std::{rc::Rc, time::SystemTime};

use eframe::{
	egui::{self, Context},
	epi,
};

use crate::{
	data::{Page, Stored},
	image::from_dynamic_image,
	threading,
};

mod about;
mod bars;
mod login;
mod messages;
mod peeps;
mod sessions;
mod settings;

#[allow(clippy::module_name_repetitions)]
/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct NeosPeepsApp {
	pub stored: Stored,
	#[serde(skip)]
	pub runtime: crate::data::RuntimeOnly,
	#[serde(skip)]
	pub threads: threading::Manager,
}

impl Default for NeosPeepsApp {
	fn default() -> Self {
		use crate::data::RuntimeOnly;
		let runtime = RuntimeOnly::default();

		Self {
			stored: Stored::default(),
			runtime,
			threads: threading::Manager::default(),
		}
	}
}

impl epi::App for NeosPeepsApp {
	fn name(&self) -> &str { env!("CARGO_PKG_NAME") }

	/// Called once before the first frame.
	fn setup(
		&mut self, ctx: &Context, frame: &epi::Frame,
		storage: Option<&dyn epi::Storage>,
	) {
		crate::styling::setup_styles(ctx);

		if let Some(storage) = storage {
			*self = epi::get_value(storage, epi::APP_KEY).unwrap_or_default();

			if let Some(user_session) = self.stored.user_session.clone() {
				self.try_use_session(user_session, frame);
			}
		}

		if self.stored.check_updates
			&& self.stored.last_update_check_time
				+ std::time::Duration::from_secs(60 * 60 * 24)
				< SystemTime::now()
		{
			self.check_updates();
		}

		// Request screen updates at least once in a while
		// As normally egui doesn't update if it doesn't need to.
		let frame = std::sync::Arc::<
			std::sync::Mutex<eframe::epi::backend::FrameData>,
		>::downgrade(&frame.0);
		std::thread::spawn(move || {
			while let Some(frame_data) = frame.upgrade() {
				let frame = epi::Frame(frame_data);
				frame.request_repaint();
				std::thread::sleep(std::time::Duration::from_millis(1000));
			}
		});
	}

	fn save(&mut self, storage: &mut dyn epi::Storage) {
		epi::set_value(storage, epi::APP_KEY, self);
	}

	/// Called each time the UI needs repainting, which may be many times per
	/// second. Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`,
	/// `Window` or `Area`.
	fn update(&mut self, ctx: &Context, frame: &epi::Frame) {
		let is_authenticated =
			self.runtime.neos_api.as_ref().map_or(false, |a| a.is_authenticated());

		if self.runtime.default_profile_picture.is_none() {
			let user_img =
				image::load_from_memory(include_bytes!("../../static/user.png"))
					.expect("Failed to load image");
			self.runtime.default_profile_picture = Some(Rc::new(
				ctx.load_texture("default-pfp", from_dynamic_image(&user_img)),
			));
		}

		if is_authenticated
			&& self.runtime.last_background_refresh + self.stored.refresh_frequency
				< SystemTime::now()
		{
			self.cull_textures();
			self.runtime.last_background_refresh = SystemTime::now();
			self.refresh_friends(frame);
			self.refresh_sessions(frame);
			self.refresh_messages(frame);
		}

		self.try_recv(frame);

		egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
			self.top_bar(ui, frame);
		});

		egui::CentralPanel::default().show(ctx, |ui| {
			egui::ScrollArea::vertical().show(ui, |ui| {
				ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
					if is_authenticated {
						if self.runtime.available_update.is_some() {
							self.update_window(ctx);
						}
						if self.runtime.user_window.borrow().is_some() {
							self.user_window(ctx, frame);
						}
						if self.runtime.session_window.borrow().is_some() {
							self.session_window(ctx, frame);
						}

						match self.stored.page {
							Page::About => self.about_page(ui),
							Page::Credits => self.credits_page(ui),
							Page::License => self.license_page(ui),
							Page::Peeps => self.peeps_page(ctx, frame, ui),
							Page::Sessions => self.sessions_page(ctx, frame, ui),
							Page::Settings => self.settings_page(ui),
						}
					} else {
						match self.stored.page {
							Page::About => self.about_page(ui),
							Page::Credits => self.credits_page(ui),
							Page::License => self.license_page(ui),
							Page::Settings => self.settings_page(ui),
							_ => self.login_page(ui, frame),
						}
					}
				});
			});
		});
	}
}
