use eframe::{
	egui::{self, Button},
	epi,
};
use neos::{api_client::NeosRequestUserSessionIdentifier, NeosUserSession};
use std::{
	sync::{Arc, RwLock},
	time::{Duration, Instant},
};

mod about;
mod friends;
mod login;

#[allow(clippy::module_name_repetitions)]
/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct NeosPeepsApp {
	user_session: Arc<RwLock<Option<NeosUserSession>>>,
	identifier: NeosRequestUserSessionIdentifier,
	#[serde(skip)]
	runtime: crate::data::RuntimeOnly,
	refresh_frequency: Duration,
}

impl Default for NeosPeepsApp {
	fn default() -> Self {
		use crate::data::RuntimeOnly;
		let runtime = RuntimeOnly::default();

		Self {
			user_session: Arc::default(),
			identifier: NeosRequestUserSessionIdentifier::Username(
				String::default(),
			),
			runtime,
			refresh_frequency: Duration::from_secs(120),
		}
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

			let user_session = self.user_session.read().unwrap().clone();
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
		if !self.runtime.loading.read().unwrap().is_loading()
			&& self.runtime.neos_api.read().unwrap().is_authenticated()
			&& *self.runtime.last_friends_refresh.read().unwrap()
				+ self.refresh_frequency
				< Instant::now()
		{
			self.refresh_friends(frame.clone());
		}

		egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
			egui::menu::bar(ui, |ui| {
				if ui.button("About").clicked() {
					self.runtime.about_popup_showing =
						!self.runtime.about_popup_showing;
				}

				ui.separator();

				if !self.runtime.loading.read().unwrap().login_op()
					&& self.runtime.neos_api.read().unwrap().is_authenticated()
				{
					ui.menu_button("Account", |ui| {
						if ui
							.add_enabled(
								!self
									.runtime
									.loading
									.read()
									.unwrap()
									.is_loading(),
								Button::new("Refresh"),
							)
							.clicked()
						{
							self.refresh_friends(frame.clone());
						}
						ui.separator();
						if ui.add(Button::new("Log out")).clicked() {
							self.logout(frame.clone());
						}
					});
					ui.separator();
				}

				if ui.button("Quit").clicked() {
					frame.quit();
				}
			});
		});

		egui::CentralPanel::default().show(ctx, |ui| {
			egui::ScrollArea::vertical().show(ui, |ui| {
				ui.with_layout(
					egui::Layout::top_down(egui::Align::Center),
					|ui| {
						if self.runtime.about_popup_showing {
							self.about_page(ui);
						} else if self
							.runtime
							.neos_api
							.read()
							.unwrap()
							.is_authenticated()
						{
							self.friends_page(ui, frame);
						} else {
							self.login_page(ui, frame.clone());
						}
					},
				);
			});
		});
	}
}
