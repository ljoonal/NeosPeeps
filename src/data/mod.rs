use std::{collections::HashMap, rc::Rc};

use ahash::RandomState;
use eframe::egui::TextureHandle;

mod runtime;
mod stored;

pub use runtime::*;
pub use stored::*;

/// [`neos::AssetUrl`] ID's as keys.
pub type TexturesMap = HashMap<String, Rc<TextureHandle>, RandomState>;

pub type UserWindow =
	(neos::id::User, Option<neos::User>, Option<neos::UserStatus>);
pub type SessionWindow = (neos::id::Session, Option<neos::SessionInfo>);
