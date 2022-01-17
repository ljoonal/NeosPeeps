use std::{
	collections::HashMap,
	sync::{Arc, RwLock},
};

use eframe::epi;
use neos::{NeosFriend, NeosSession, NeosUserStatus};

use crate::image::{load_asset_pic, TextureDetails};

pub fn load_all_user_session_thumbnails(
	sessions: &[NeosSession],
	pics: &Arc<RwLock<HashMap<String, Option<TextureDetails>>>>,
	frame: &epi::Frame,
) {
	use rayon::prelude::*;

	sessions.par_iter().for_each(|session| {
		if let Some(url) = &session.thumbnail {
			load_asset_pic(url, pics.clone(), frame);
		}
	});
}

pub fn find_focused_session<'a>(
	id: &neos::id::User,
	user_status: &'a NeosUserStatus,
) -> Option<&'a NeosSession> {
	use rayon::prelude::*;

	user_status.active_sessions.par_iter().find_any(|session| {
		session
			.session_users
			.par_iter()
			.find_any(|user| match &user.user_id {
				Some(user_id) => user_id == id && user.is_present,
				None => false,
			})
			.is_some()
	})
}

pub fn unload_unused_session_thumbnails(
	pics: &mut HashMap<String, Option<TextureDetails>>,
	friends: &[NeosFriend],
) {
	use rayon::prelude::*;
	pics.retain(|id, pic| {
		pic.is_none() // Prevent multiple fetches.
			|| friends
				.par_iter()
				.find_any(|friend| has_session_thumbnail_asset(friend, id))
				.is_some()
	});
}

fn has_session_thumbnail_asset(friend: &NeosFriend, asset_id: &str) -> bool {
	true // TODO
}
