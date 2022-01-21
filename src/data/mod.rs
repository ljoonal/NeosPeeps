use std::{collections::HashMap, rc::Rc};

use ahash::RandomState;
use neos::{NeosSession, NeosUser, NeosUserStatus};

use crate::image::TextureDetails;

mod runtime;
mod stored;

pub use runtime::*;
pub use stored::*;

/// [`neos::AssetUrl`] ID's as keys.
pub type TexturesMap = HashMap<String, Rc<TextureDetails>, RandomState>;

pub type UserWindow =
	(neos::id::User, Option<NeosUser>, Option<NeosUserStatus>);
pub type SessionWindow = (neos::id::Session, Option<NeosSession>);
