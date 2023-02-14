//! Styles for the application

use eframe::egui::{
	Color32,
	Context,
	FontData,
	FontDefinitions,
	FontFamily,
	Stroke,
	Style,
	TextStyle,
};

pub fn setup_styles(ctx: &Context) {
	let mut style = (*ctx.style()).clone();
	setup_style(ctx, &mut style);
	setup_fonts(ctx, &mut style);

	ctx.set_style(style);
}

fn setup_style(_ctx: &Context, style: &mut Style) {
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

	style.visuals.widgets.open.weak_bg_fill = bg_faint;
	style.visuals.widgets.open.bg_fill = bg_faint;

	style.visuals.widgets.active.weak_bg_fill = Color32::BLACK;
	style.visuals.widgets.active.bg_fill = Color32::BLACK;
	style.visuals.widgets.active.fg_stroke.color = fg;

	style.visuals.widgets.hovered.weak_bg_fill = Color32::BLACK;
	style.visuals.widgets.hovered.bg_fill = Color32::BLACK;
	style.visuals.widgets.hovered.bg_stroke.color = fg;
	style.visuals.widgets.hovered.fg_stroke.color = fg;

	style.visuals.widgets.inactive.weak_bg_fill = Color32::BLACK;
	// Because scrollbar also uses this need to have it be a bit different from
	// extreme_bg_color which is the BG of it.
	style.visuals.widgets.inactive.bg_fill = Color32::from_rgb(10, 10, 14);
	style.visuals.widgets.inactive.bg_stroke =
		Stroke { color: Color32::BLACK, width: 2.0 };
	style.visuals.widgets.inactive.fg_stroke.color = fg;

	style.visuals.widgets.noninteractive.fg_stroke.color = fg;

	style.visuals.panel_fill = bg;
	style.visuals.window_fill = bg;
	style.visuals.widgets.noninteractive.bg_fill = bg;
	style.visuals.widgets.noninteractive.weak_bg_fill = bg;

	style.visuals.selection.bg_fill = Color32::BLACK;
	style.visuals.selection.stroke.color = fg;

	style.spacing.item_spacing.y = 8_f32;
	style.spacing.button_padding.y = 5_f32;
}

fn setup_fonts(ctx: &Context, style: &mut Style) {
	const JP_FONT_ERR: &str = "This might cause some characters, like japanese ones, to not render properly.";
	const JP_FONT: &str = "Noto Sans CJK JP font";

	let mut fonts = FontDefinitions::default();

	if let Some(font_id) = style.text_styles.get_mut(&TextStyle::Heading) {
		font_id.size = 34_f32;
	}

	if let Some(font_id) = style.text_styles.get_mut(&TextStyle::Body) {
		font_id.size = 24_f32;
	}
	if let Some(font_id) = style.text_styles.get_mut(&TextStyle::Button) {
		font_id.size = 26_f32;
	}

	if let Some(font_id) = style.text_styles.get_mut(&TextStyle::Monospace) {
		font_id.size = 22_f32;
	}

	if let Some(font_id) = style.text_styles.get_mut(&TextStyle::Small) {
		font_id.size = 16_f32;
	}

	fonts.font_data.insert(
		"raleway".to_owned(),
		FontData::from_static(include_bytes!("../static/Raleway.ttf")),
	);
	fonts
		.families
		.get_mut(&FontFamily::Proportional)
		.unwrap()
		.insert(0, "raleway".to_owned());
	fonts
		.families
		.get_mut(&FontFamily::Monospace)
		.unwrap()
		.push("raleway".to_owned());

	#[cfg(feature = "font-kit")]
	{
		use font_kit::family_name::FamilyName;
		use font_kit::properties::{Properties, Style, Weight};
		use font_kit::source::SystemSource;
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
						.families
						.get_mut(&FontFamily::Proportional)
						.unwrap()
						.push("noto-cjk-jp".to_owned());
					fonts
						.families
						.get_mut(&FontFamily::Monospace)
						.unwrap()
						.push("noto-cjk-jp".to_owned());
				} else {
					eprintln!("Failed to load the data of {}. {}", JP_FONT, JP_FONT_ERR);
				}
			} else {
				eprintln!("Failed to load {}. {}", JP_FONT, JP_FONT_ERR);
			}
		} else {
			eprintln!("Couldn't find {}. {}", JP_FONT, JP_FONT_ERR);
		}
	}

	ctx.set_fonts(fonts);
}
