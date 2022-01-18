use crate::{
	data::{Page, Stored},
	image::TextureDetails,
};
use eframe::{egui, epi};
use std::{sync::Arc, time::SystemTime};

mod about;
mod bar;
mod friends;
mod login;
mod sessions;
mod settings;

#[allow(clippy::module_name_repetitions)]
/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct NeosPeepsApp {
	stored: Stored,
	#[serde(skip)]
	runtime: crate::data::RuntimeOnly,
}

impl Default for NeosPeepsApp {
	fn default() -> Self {
		use crate::data::RuntimeOnly;
		let runtime = RuntimeOnly::default();

		Self { stored: Stored::default(), runtime }
	}
}

impl epi::App for NeosPeepsApp {
	fn name(&self) -> &str {
		env!("CARGO_PKG_NAME")
	}

	/// Called once before the first frame.
	fn setup(
		&mut self,
		ctx: &egui::CtxRef,
		frame: &epi::Frame,
		storage: Option<&dyn epi::Storage>,
	) {
		crate::styling::setup_styles(ctx);

		if let Some(storage) = storage {
			*self = epi::get_value(storage, epi::APP_KEY).unwrap_or_default();

			let user_session = self.stored.user_session.read().unwrap().clone();
			if let Some(user_session) = user_session {
				self.try_use_session(user_session, frame.clone());
			}
		}
	}

	fn save(&mut self, storage: &mut dyn epi::Storage) {
		epi::set_value(storage, epi::APP_KEY, self);
	}

	/// Called each time the UI needs repainting, which may be many times per
	/// second. Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`,
	/// `Window` or `Area`.
	fn update(&mut self, ctx: &egui::CtxRef, frame: &epi::Frame) {
		let is_authenticated =
			self.runtime.neos_api.read().unwrap().is_authenticated();
		let is_loading = self.runtime.loading.read().unwrap().is_loading();

		if self.runtime.default_profile_picture.is_none() {
			let user_img = image::load_from_memory(include_bytes!(
				"../../static/user.png"
			))
			.expect("Failed to load image");
			self.runtime.default_profile_picture = Some(Arc::new(
				TextureDetails::from_image(frame.clone(), &user_img),
			));
		}

		if !is_loading
			&& is_authenticated
			&& *self.runtime.last_background_refresh.read().unwrap()
				+ self.stored.refresh_frequency
				< SystemTime::now()
		{
			self.runtime.cull_textures();
			*self.runtime.last_background_refresh.clone().write().unwrap() =
				SystemTime::now();
			self.refresh_friends(frame.clone());
			self.refresh_sessions(frame.clone());
		}

		egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
			self.top_bar(ui, frame);
		});

		egui::CentralPanel::default().show(ctx, |ui| {
			if is_loading {
				ui.vertical_centered_justified(|ui| {
					ui.label("Loading...");
				});
			}

			egui::ScrollArea::vertical().show(ui, |ui| {
				ui.with_layout(
					egui::Layout::top_down(egui::Align::Center),
					|ui| {
						if is_authenticated {
							match self.stored.page {
								Page::About => self.about_page(ui),
								Page::Friends => self.friends_page(ui, frame),
								Page::Sessions => self.sessions_page(ui, frame),
								Page::Settings => self.settings_page(ui, frame),
							}
						} else {
							match self.stored.page {
								Page::About => self.about_page(ui),
								Page::Settings => self.settings_page(ui, frame),
								_ => self.login_page(ui, frame.clone()),
							}
						}
					},
				);
			});
		});
	}
}
