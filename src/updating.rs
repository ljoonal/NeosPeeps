use std::time::SystemTime;

use eframe::egui::{Context, Id, Window};
use serde::Deserialize;

use crate::app::NeosPeepsApp;

const UPDATE_CHECK_BASE: &str =
	"https://git.ljoonal.xyz/api/v1/repos/ljoonal/NeosPeeps/";

#[derive(Deserialize)]
pub struct GiteaReleasesResponse {
	pub html_url: String,
	pub tag_name: String,
}

impl NeosPeepsApp {
	pub fn update_window(&mut self, ctx: &Context) {
		let mut open = true;
		if let Some(rel) = &self.runtime.available_update {
			Window::new("Updates available")
				.id(Id::new("update_window"))
				.open(&mut open)
				.show(ctx, |ui| {
					ui.heading(&rel.tag_name);
					ui.label("The new version is available from:");
					ui.hyperlink(&rel.html_url);
				});
		}

		if !open {
			self.runtime.available_update = None;
		}
	}

	pub fn check_updates(&mut self) {
		self.stored.last_update_check_time = SystemTime::now();
		let update_check_sender = self.threads.channels.update_check_sender();

		// .... This is just clearer, plain and simple.
		#[allow(clippy::option_if_let_else)]
		std::thread::spawn(move || {
			let res = latest_version_request();
			if let Ok(rel) = res {
				if let Some(rel_v) = rel.tag_name.strip_prefix('v') {
					if rel_v != env!("CARGO_PKG_VERSION") {
						update_check_sender.send(rel).unwrap();
					}
				} else {
					eprintln!("Update check returned wrongly formatted tag name!");
				}
			} else if let Err(err) = res {
				eprintln!("{err}");
			}
		});
	}
}

fn latest_version_request() -> Result<GiteaReleasesResponse, String> {
	let url = UPDATE_CHECK_BASE.to_owned() + "releases";

	let res = minreq::get(url.to_string())
		.with_header("User-Agent", crate::USER_AGENT)
		.with_param("draft", "false")
		.with_param("pre-release", "false")
		.with_param("limit", "1")
		.send()
		.map_err(|err| {
			format!("Failed to send update check request {url:?} - {err}")
		})?;

	if res.status_code < 200 || res.status_code >= 300 {
		return Err(format!(
			"Update check request status indicated failure: {}",
			res.status_code,
		));
	}

	let mut data: Vec<GiteaReleasesResponse> =
		res.json().map_err(|err| format!("Couldn't parse response: {err}"))?;

	data.pop().ok_or_else(|| "No releases returned!".to_owned())
}
