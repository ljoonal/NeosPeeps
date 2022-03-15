//! The friends page of the app
use eframe::{
	egui::{Context, Grid, Key, Label, Layout, ScrollArea, Sense, TextEdit, Ui},
	emath::Align,
	epaint::Vec2,
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
					neos::MessageContents::SessionInvite(session) => {
						ui.label("Invited to session:");
						if ui
							.add(
								Label::new(session.stripped_name())
									.wrap(true)
									.sense(Sense::click()),
							)
							.clicked()
						{
							*self.runtime.session_window.borrow_mut() =
								Some((session.id.clone(), Some(*session.clone())));
						}
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

	pub fn chat_page(&mut self, ctx: &Context, frame: &epi::Frame, ui: &mut Ui) {
		use rayon::prelude::*;

		if ui.button("Back").clicked() {
			*self.runtime.open_chat.borrow_mut() = None;
		}

		let mut send_message = false;

		let mut opt = self.runtime.open_chat.borrow_mut();
		if let Some((user_id, typed_msg)) = &mut *opt {
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

				ui.with_layout(Layout::bottom_up(Align::Center), |ui| {
					ui.set_height(ui.available_height());
					ui.allocate_ui_with_layout(
						Vec2::new(ui.available_width(), 36f32),
						Layout::right_to_left(),
						|ui| {
							if ui.button("Send").clicked() {
								send_message = true;
							}
							let response = ui.add_sized(
								ui.available_size(),
								TextEdit::singleline(typed_msg)
									.desired_width(ui.available_width()),
							);
							if response.lost_focus() && ui.input().key_pressed(Key::Enter) {
								send_message = true;
							}
						},
					);

					ui.with_layout(Layout::top_down(Align::Center), |ui| {
						ui.set_height(ui.available_height());

						if let Some(messages) = self.runtime.messages.get(&friend.id) {
							let mut messages: Vec<&neos::Message> =
								messages.values().collect();
							messages.par_sort_unstable_by_key(|m| m.send_time);

							ScrollArea::vertical()
								.max_height(ui.available_height())
								.stick_to_bottom()
								.show_rows(
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
													self.message_row(
														ctx, frame, ui, width, friend, message,
													);
													ui.end_row();
												}
											});
									},
								);
						} else {
							ui.label("No messages yet");
						}
					});
				});
			} else {
				ui.heading("Peep not found");
			}
		} else {
			ui.heading("Internal error");
			ui.label("Chat page is being show even though there is no id for a chat");
		}

		if send_message && !self.threads.loading.messages.get() {
			let mut taken_opt: Option<(neos::id::User, String)> = None;

			if let Some(opt) = &mut *opt {
				taken_opt = Some((opt.0.clone(), std::mem::take(&mut opt.1)));
			}

			drop(opt);

			if let Some((user_id, typed_msg)) = taken_opt {
				let message = neos::Message::new(
					neos::MessageContents::Text(typed_msg),
					self.stored.user_session.as_ref().unwrap().user_id.clone(),
					user_id,
				);
				self.send_message(frame, message);
			}
		}
	}
}
