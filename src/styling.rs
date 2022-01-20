//! Styles for the application

use eframe::egui::{
	self,
	Color32,
	FontData,
	FontDefinitions,
	FontFamily,
	Stroke,
	TextStyle,
};

pub fn setup_styles(ctx: &egui::CtxRef) {
	setup_style(ctx);
	setup_fonts(ctx);
}

fn setup_style(ctx: &egui::CtxRef) {
	let mut style = (*ctx.style()).clone();

	#[cfg(debug_assertions)]
	{
		style.debug.debug_on_hover = true;
	}

	let bg = Color32::from_rgb(7, 7, 7);
	let bg_faint = Color32::from_rgb(10, 10, 10);
	let bg_interact = Color32::BLACK;
	let fg = Color32::from_rgb(240, 240, 240);

	style.visuals.dark_mode = true;
	style.visuals.widgets.noninteractive.fg_stroke.color = fg;
	style.visuals.widgets.active.fg_stroke.color = fg;
	style.visuals.widgets.inactive.bg_stroke =
		Stroke { width: 1., color: Color32::from_rgb(100, 100, 100) };

	style.visuals.faint_bg_color = bg_faint;
	style.visuals.extreme_bg_color = bg_interact;
	style.visuals.code_bg_color = fg;

	style.visuals.widgets.noninteractive.bg_fill = bg;
	style.visuals.widgets.open.bg_fill = bg_faint;
	style.visuals.widgets.inactive.bg_fill = bg_interact;
	style.visuals.widgets.active.bg_fill = bg_interact;
	style.visuals.widgets.hovered.bg_fill = bg_interact;

	style.spacing.item_spacing.y = 8_f32;
	style.spacing.button_padding.y = 5_f32;

	ctx.set_style(style);
}

fn setup_fonts(ctx: &egui::CtxRef) {
	let mut fonts = FontDefinitions::default();

	if let Some((_, size)) = fonts.family_and_size.get_mut(&TextStyle::Heading) {
		*size = 34_f32;
	}

	if let Some((_, size)) = fonts.family_and_size.get_mut(&TextStyle::Body) {
		*size = 24_f32;
	}
	if let Some((_, size)) = fonts.family_and_size.get_mut(&TextStyle::Button) {
		*size = 26_f32;
	}

	if let Some((_, size)) = fonts.family_and_size.get_mut(&TextStyle::Monospace)
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
	fonts.font_data.insert(
		"noto-cjk-jp".to_owned(),
		FontData::from_static(include_bytes!("../static/NotoSansCJKjp-VF.ttf")),
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

	fonts
		.fonts_for_family
		.get_mut(&FontFamily::Proportional)
		.unwrap()
		.push("noto-cjk-jp".to_owned());
	fonts
		.fonts_for_family
		.get_mut(&FontFamily::Monospace)
		.unwrap()
		.push("noto-cjk-jp".to_owned());

	ctx.set_fonts(fonts);
}
