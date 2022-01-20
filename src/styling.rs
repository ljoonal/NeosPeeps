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
	use font_kit::family_name::FamilyName;
	use font_kit::properties::{Properties, Style, Weight};
	use font_kit::source::SystemSource;

	const JP_FONT_ERR: &str = "This might cause some characters, like japanese ones, to not render properly.";
	const JP_FONT: &str = "Noto Sans CJK JP font";

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

	if let Ok(font_handle) = SystemSource::new().select_best_match(
		&[FamilyName::Title("Noto Sans CJK JP".to_string())],
		Properties::new().style(Style::Normal).weight(Weight::LIGHT),
	) {
		if let Ok(font) = font_handle.load() {
			if let Some(font_data) = font.copy_font_data() {
				fonts.font_data.insert(
					"noto-cjk-jp".to_owned(),
					FontData::from_owned((*font_data).clone()),
				);
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
			} else {
				println!("Failed to load the data of {}. {}", JP_FONT, JP_FONT_ERR);
			}
		} else {
			println!("Failed to load {}. {}", JP_FONT, JP_FONT_ERR);
		}
	} else {
		println!("Couldn't find {}. {}", JP_FONT, JP_FONT_ERR);
	}

	ctx.set_fonts(fonts);
}
