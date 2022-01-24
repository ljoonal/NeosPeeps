use std::{collections::HashMap, rc::Rc};

use ahash::RandomState;
use eframe::egui::TextureHandle;
use neos::{NeosSession, NeosUser, NeosUserStatus};

mod runtime;
mod stored;

pub use runtime::*;
pub use stored::*;

/// [`neos::AssetUrl`] ID's as keys.
pub type TexturesMap = HashMap<String, Rc<TextureHandle>, RandomState>;

pub type UserWindow =
	(neos::id::User, Option<NeosUser>, Option<NeosUserStatus>);
pub type SessionWindow = (neos::id::Session, Option<NeosSession>);
