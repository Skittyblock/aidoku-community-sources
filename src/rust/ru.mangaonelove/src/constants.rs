use aidoku::error::{AidokuError, AidokuErrorKind, NodeError};

pub const MANGA_DIR: &str = "manga";
pub const PAGE_DIR: &str = "page";
pub const SEARCH_OFFSET_STEP: i32 = 10;

pub const PARSING_ERROR: AidokuError = AidokuError {
	reason: AidokuErrorKind::NodeError(NodeError::ParseError),
};

pub const UNIMPLEMENTED_ERROR: AidokuError = AidokuError {
	reason: AidokuErrorKind::Unimplemented,
};
