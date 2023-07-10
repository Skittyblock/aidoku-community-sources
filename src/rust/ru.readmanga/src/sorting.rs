use core::fmt::Display;

use aidoku::Listing;

#[derive(Debug, Default)]
pub enum Sorting {
	#[default]
	Rating,
	Popular,
	UpdatedRecently,
}

impl Sorting {
	pub fn from_listing(listing: &Listing) -> Self {
		match listing.name.as_str() {
			"Rating" => Self::Rating,
			"Popular" => Self::Popular,
			"Updated Recently" => Self::UpdatedRecently,
			_ => Self::Rating,
		}
	}
}

impl Display for Sorting {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		match self {
			Sorting::Rating => write!(f, "RATING"),
			Sorting::Popular => write!(f, "POPULARITY"),
			Sorting::UpdatedRecently => write!(f, "DATE_UPDATE"),
		}
	}
}
