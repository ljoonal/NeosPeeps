use std::time::Duration;

use eframe::egui::{Slider, Ui};

use super::NeosPeepsApp;
use crate::data::Page;

impl NeosPeepsApp {
	pub fn settings_page(&mut self, ui: &mut Ui) {
		ui.heading("Settings");

		{
			let mut refresh_freq: u64 = self.stored.refresh_frequency.as_secs();
			if ui
				.add(
					Slider::new(&mut refresh_freq, 5..=900)
						.text("Refresh frequency")
						.suffix("s"),
				)
				.changed()
			{
				self.stored.refresh_frequency = Duration::from_secs(refresh_freq);
			}
		}

		ui.add(
			Slider::new(&mut self.stored.row_height, 100_f32..=500_f32)
				.fixed_decimals(0)
				.clamp_to_range(false)
				.text("Row height"),
		);

		ui.add(
			Slider::new(&mut self.stored.col_min_width, 100_f32..=500_f32)
				.fixed_decimals(0)
				.clamp_to_range(false)
				.text("Column min width"),
		);

		if ui.button("Back").clicked() {
			self.stored.page = Page::Peeps;
		}
	}
}
