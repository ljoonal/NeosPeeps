//! The friends page of the app
use eframe::{
	egui::{Context, Grid, Layout, ScrollArea, Ui},
	emath::Align,
	epi,
};

use super::NeosPeepsApp;

impl NeosPeepsApp {
	fn message_row(
		&self, ctx: &Context, frame: &epi::Frame, ui: &mut Ui, width: f32,
		friend: &neos::Friend, message: &neos::Message,
	) {
		ui.with_layout(Layout::top_down(Align::LEFT), |ui| {
			ui.set_max_width(self.stored.col_min_width.min(
				width - self.stored.row_height - ui.style().spacing.item_spacing.x,
			));

			ui.label(
				message.send_time.format(&self.stored.datetime_format).to_string(),
			);
			if message.sender_id == friend.id {
				ui.label(&friend.username);
			} else {
				ui.label("You");
			}
		});

		ui.with_layout(Layout::left_to_right(), |ui| {
			ui.set_width(ui.available_width());

			ui.separator();

			ui.with_layout(Layout::top_down(Align::LEFT), |ui| {
				match &message.content {
					neos::MessageContents::Text(content) => {
						ui.label(content);
					}
					neos::MessageContents::SessionInvite(inv) => {
						ui.label("Session invite");
						ui.label(inv.stripped_name());
					}
					neos::MessageContents::CreditTransfer(transaction) => {
						ui.horizontal(|ui| {
							ui.label("Sent ");
							ui.label(&transaction.token);
							ui.label(":");
						});
						ui.label(&transaction.amount.to_string());
					}
					neos::MessageContents::Sound(obj) => {
						ui.label("TODO!!! Sound message");
					}
					neos::MessageContents::Object(obj) => {
						ui.label("TODO!!! obj");
					}
					neos::MessageContents::SugarCubes(obj) => {
						ui.label("Kofi tipping transaction");
						ui.label("Note: unsupported msg type");
					}
				}
			});
		});
	}

	pub fn chat_page(
		&mut self, ctx: &Context, frame: &epi::Frame, ui: &mut Ui,
		user_id: &neos::id::User,
	) {
		use rayon::prelude::*;

		if ui.button("Back").clicked() {
			*self.runtime.open_chat.borrow_mut() = None;
		}
		if let Some(friend) =
			self.runtime.friends.par_iter().find_any(|friend| &friend.id == user_id)
		{
			self.clickable_username(
				ui,
				frame,
				&friend.id,
				&friend.username,
				None,
				None,
			);

			if let Some(messages) = self.runtime.messages.get(&friend.id) {
				let mut messages: Vec<&neos::Message> = messages.values().collect();
				messages.par_sort_unstable_by_key(|m| m.send_time);

				ScrollArea::vertical().stick_to_bottom().show_rows(
					ui,
					self.stored.row_height,
					messages.len(),
					|ui, row_range| {
						let width = ui.available_width();
						Grid::new("messages_list")
							.start_row(row_range.start)
							.striped(true)
							.min_row_height(self.stored.row_height)
							.num_columns(2)
							.show(ui, |ui| {
								for row in row_range {
									let message = messages[row];
									self.message_row(ctx, frame, ui, width, friend, message);
									ui.end_row();
								}
							});
					},
				);
			}
		} else {
			ui.heading("Peep not found");
		}
	}
}
