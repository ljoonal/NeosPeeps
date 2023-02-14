use std::{rc::Rc, time::SystemTime};

use eframe::egui::{self, Context, TextureOptions};

use crate::{
	data::{Page, Stored},
	image::from_dynamic_image,
	threading,
};

mod about;
mod bars;
mod chat;
mod login;
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

impl eframe::App for NeosPeepsApp {
	fn save(&mut self, storage: &mut dyn eframe::Storage) {
		eframe::set_value(storage, eframe::APP_KEY, self);
	}

	/// Called each time the UI needs repainting, which may be many times per
	/// second. Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`,
	/// `Window` or `Area`.
	fn update(&mut self, ctx: &Context, frame: &mut eframe::Frame) {
		let is_authenticated =
			self.runtime.neos_api.as_ref().map_or(false, |a| a.is_authenticated());

		if self.runtime.default_profile_picture.is_none() {
			let user_img =
				image::load_from_memory(include_bytes!("../../static/user.png"))
					.expect("Failed to load image");
			self.runtime.default_profile_picture = Some(Rc::new(ctx.load_texture(
				"default-pfp",
				from_dynamic_image(&user_img),
				TextureOptions::LINEAR,
			)));
		}

		if is_authenticated
			&& self.runtime.last_background_refresh + self.stored.refresh_frequency
				< SystemTime::now()
		{
			self.cull_textures();
			self.runtime.last_background_refresh = SystemTime::now();
			self.refresh_friends(ctx);
			self.refresh_sessions(ctx);
			self.refresh_messages(ctx);
		}

		self.try_recv(ctx);

		egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
			self.top_bar(ui, ctx, frame);
		});

		egui::CentralPanel::default().show(ctx, |ui| {
			egui::ScrollArea::vertical().show(ui, |ui| {
				ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
					if is_authenticated {
						if self.runtime.available_update.is_some() {
							self.update_window(ctx);
						}
						if self.runtime.user_window.borrow().is_some() {
							self.user_window(ctx);
						}
						if self.runtime.session_window.borrow().is_some() {
							self.session_window(ctx);
						}

						match self.stored.page {
							Page::About => self.about_page(ui),
							Page::Credits => self.credits_page(ui),
							Page::License => self.license_page(ui),
							Page::Peeps => self.peeps_page(ctx, ui),
							Page::Sessions => self.sessions_page(ctx, ui),
							Page::Settings => self.settings_page(ui),
						}
					} else {
						match self.stored.page {
							Page::About => self.about_page(ui),
							Page::Credits => self.credits_page(ui),
							Page::License => self.license_page(ui),
							Page::Settings => self.settings_page(ui),
							_ => self.login_page(ui, ctx),
						}
					}
				});
			});
		});
	}
}

impl NeosPeepsApp {
	pub fn new(creation_ctx: &eframe::CreationContext<'_>) -> Self {
		let mut app = Self::default();

		crate::styling::setup_styles(&creation_ctx.egui_ctx);

		if let Some(storage) = creation_ctx.storage {
			app = eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();

			if let Some(user_session) = app.stored.user_session.clone() {
				app.try_use_session(user_session, &creation_ctx.egui_ctx);
			}
		}

		if app.stored.check_updates
			&& app.stored.last_update_check_time
				+ std::time::Duration::from_secs(60 * 60 * 24)
				< SystemTime::now()
		{
			app.check_updates();
		}

		let min_fps_ctx = creation_ctx.egui_ctx.clone();
		std::thread::spawn(move || {
			loop {
				std::thread::sleep(std::time::Duration::from_millis(1000));
				min_fps_ctx.request_repaint();
			}
		});

		app
	}
}
