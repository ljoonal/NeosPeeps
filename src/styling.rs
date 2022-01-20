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

	let bg = Color32::from_rgb(8, 3, 0);
	let bg_faint = Color32::from_rgb(10, 5, 0);
	let fg = Color32::from_rgb(240, 240, 240);

	style.visuals.dark_mode = true;
	style.visuals.faint_bg_color = bg_faint;
	style.visuals.extreme_bg_color = Color32::BLACK;
	style.visuals.code_bg_color = Color32::BLACK;

	style.visuals.widgets.open.bg_fill = bg_faint;

	style.visuals.widgets.active.bg_fill = Color32::BLACK;
	style.visuals.widgets.active.fg_stroke.color = fg;

	style.visuals.widgets.hovered.bg_fill = Color32::BLACK;
	style.visuals.widgets.hovered.bg_stroke.color = fg;
	style.visuals.widgets.hovered.fg_stroke.color = fg;

	// Because scrollbar also uses this need to have it be a bit different from
	// extreme_bg_color which is the BG of it.
	style.visuals.widgets.inactive.bg_fill = Color32::from_rgb(10, 10, 14);
	style.visuals.widgets.inactive.bg_stroke =
		Stroke { color: Color32::BLACK, width: 2.0 };
	style.visuals.widgets.inactive.fg_stroke.color = fg;

	style.visuals.widgets.noninteractive.fg_stroke.color = fg;
	style.visuals.widgets.noninteractive.bg_fill = bg;

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
		"raleway".to_owned(),
		FontData::from_static(include_bytes!("../static/Raleway.ttf")),
	);
	//	fonts.font_data.insert(
	//		"noto-cjk-jp".to_owned(),
	//		FontData::from_static(include_bytes!("../static/NotoSansCJKjp-VF.ttf")),
	//	);
	fonts
		.fonts_for_family
		.get_mut(&FontFamily::Proportional)
		.unwrap()
		.insert(0, "raleway".to_owned());
	fonts
		.fonts_for_family
		.get_mut(&FontFamily::Monospace)
		.unwrap()
		.push("raleway".to_owned());

	//	fonts
	//		.fonts_for_family
	//		.get_mut(&FontFamily::Proportional)
	//		.unwrap()
	//		.push("noto-cjk-jp".to_owned());
	//	fonts
	//		.fonts_for_family
	//		.get_mut(&FontFamily::Monospace)
	//		.unwrap()
	//		.push("noto-cjk-jp".to_owned());

	ctx.set_fonts(fonts);
}
