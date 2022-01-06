use crate::{api::NeosApi, image::ImageDetails};
use eframe::{
	egui::{self, Button},
	epi,
};
use std::sync::{Arc, RwLock};
use std::thread;

mod about;
mod friends;
mod login;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(Default, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct NeosPeepsApp {
	username: String,
	#[serde(skip)]
	password: String,
	#[serde(skip)]
	loading_data: Arc<RwLock<bool>>,
	#[serde(skip)]
	logging_in: Arc<RwLock<bool>>,
	#[serde(skip)]
	default_profile_picture: Option<ImageDetails>,
	#[serde(skip)]
	about_popup_showing: bool,
	neos_api: Arc<RwLock<Option<NeosApi>>>,
	#[serde(skip)]
	friends: Arc<RwLock<Vec<neos::NeosFriend>>>,
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

			if self.neos_api.read().unwrap().is_some() {
				*self.logging_in.write().unwrap() = true;

				let neos_api_arc = self.neos_api.clone();
				let logging_in = self.logging_in.clone();
				let frame = frame.clone();
				thread::spawn(move || {
					if let Some(neos_api) = &*neos_api_arc.read().unwrap() {
						match neos_api.extend_session() {
							Ok(_) => {
								println!("NeosApi: {:?}", neos_api);
							}
							Err(e) => {
								*neos_api_arc.write().unwrap() = None;
								println!("Error with Neos API: {}", e);
							}
						}
					}
					*logging_in.write().unwrap() = false;
					frame.request_repaint();
				});
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
				if ui.button("About").clicked() {
					self.about_popup_showing = !self.about_popup_showing;
				}

				ui.separator();

				if !*self.logging_in.read().unwrap()
					&& self.neos_api.read().unwrap().is_some()
				{
					if ui.add(Button::new("Refresh")).clicked() {
						self.refresh_friends(frame.clone());
					}
					ui.separator();
					if ui.add(Button::new("Log out")).clicked() {
						*self.neos_api.write().unwrap() = None;
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
					egui::Layout::top_down_justified(egui::Align::Center),
					|ui| {
						if self.about_popup_showing {
							self.about_page(ui);
						} else if self.neos_api.read().unwrap().is_some() {
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
