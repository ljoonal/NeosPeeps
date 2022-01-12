use crate::image::TextureDetails;
use eframe::{
	egui::{self, warn_if_debug_build, Button},
	epi,
};
use neos::{
	api_client::{
		AnyNeos,
		NeosRequestUserSessionIdentifier,
		NeosUnauthenticated,
	},
	NeosUserSession,
};
use std::sync::{Arc, RwLock};

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
	password: String,
	#[serde(skip)]
	totp: String,
	#[serde(skip)]
	loading_data: Arc<RwLock<bool>>,
	#[serde(skip)]
	logging_in: Arc<RwLock<bool>>,
	#[serde(skip)]
	default_profile_picture: Option<TextureDetails>,
	#[serde(skip)]
	about_popup_showing: bool,
	#[serde(skip)]
	neos_api: Arc<RwLock<AnyNeos>>,
	#[serde(skip)]
	friends: Arc<RwLock<Vec<neos::NeosFriend>>>,
}

impl Default for NeosPeepsApp {
	fn default() -> Self {
		let api = NeosUnauthenticated::new(crate::USER_AGENT.to_owned());

		Self {
			user_session: Arc::default(),
			identifier: NeosRequestUserSessionIdentifier::Username(
				String::default(),
			),
			totp: String::default(),
			password: String::default(),
			loading_data: Arc::default(),
			logging_in: Arc::default(),
			default_profile_picture: Option::default(),
			about_popup_showing: Default::default(),
			neos_api: Arc::new(RwLock::new(AnyNeos::Unauthenticated(api))),
			friends: Arc::default(),
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
		egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
			egui::menu::bar(ui, |ui| {
				warn_if_debug_build(ui);
				if ui.button("About").clicked() {
					self.about_popup_showing = !self.about_popup_showing;
				}

				ui.separator();

				if !*self.logging_in.read().unwrap()
					&& self.neos_api.read().unwrap().is_authenticated()
				{
					if ui.add(Button::new("Refresh")).clicked() {
						self.refresh_friends(frame.clone());
					}
					ui.separator();
					if ui.add(Button::new("Log out")).clicked() {
						self.logout(frame.clone());
					}
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
						if self.about_popup_showing {
							self.about_page(ui);
						} else if self
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
