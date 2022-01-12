//! Styles for the application

use eframe::egui::{self, FontData, FontDefinitions, FontFamily, TextStyle};

pub fn setup_styles(ctx: &egui::CtxRef) {
	let mut style = (*ctx.style()).clone();

	#[cfg(debug_assertions)]
	{
		style.debug.debug_on_hover = true;
	}

	style.spacing.item_spacing.y = 8_f32;
	style.spacing.button_padding.y = 5_f32;

	ctx.set_style(style);

	setup_fonts(ctx);
}

fn setup_fonts(ctx: &egui::CtxRef) {
	let mut fonts = FontDefinitions::default();

	if let Some((_, size)) = fonts.family_and_size.get_mut(&TextStyle::Heading)
	{
		*size = 34_f32;
	}

	if let Some((_, size)) = fonts.family_and_size.get_mut(&TextStyle::Body) {
		*size = 24_f32;
	}
	if let Some((_, size)) = fonts.family_and_size.get_mut(&TextStyle::Button) {
		*size = 26_f32;
	}

	if let Some((_, size)) =
		fonts.family_and_size.get_mut(&TextStyle::Monospace)
	{
		*size = 22_f32;
	}

	if let Some((_, size)) = fonts.family_and_size.get_mut(&TextStyle::Small) {
		*size = 16_f32;
	}

	fonts.font_data.insert(
		"zillaslab-regular".to_owned(),
		FontData::from_static(include_bytes!("../static/Raleway.ttf")),
	);
	fonts
		.fonts_for_family
		.get_mut(&FontFamily::Proportional)
		.unwrap()
		.insert(0, "zillaslab-regular".to_owned());
	fonts
		.fonts_for_family
		.get_mut(&FontFamily::Monospace)
		.unwrap()
		.push("zillaslab-regular".to_owned());

	ctx.set_fonts(fonts);
}
