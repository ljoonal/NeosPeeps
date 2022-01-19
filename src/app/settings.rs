use std::time::Duration;

use crate::data::Page;

use super::NeosPeepsApp;
use eframe::{
	egui::{Slider, Ui},
	epi,
};

impl NeosPeepsApp {
	pub fn settings_page(&mut self, ui: &mut Ui, _frame: &epi::Frame) {
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
				self.stored.refresh_frequency =
					Duration::from_secs(refresh_freq);
			}
		}

		ui.add(
			Slider::new(&mut self.stored.row_height, 100_f32..=500_f32)
				.fixed_decimals(0)
				.text("Row height"),
		);

		if ui.button("Back").clicked() {
			self.stored.page = Page::Peeps;
		}
	}
}
