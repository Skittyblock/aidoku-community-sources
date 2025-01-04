pub enum SortOptions {
	BestMatch,
	Alphabet,
	Popularity,
	Subscribers,
	RecentlyAdded,
	LatestUpdates,
}

impl From<i32> for SortOptions {
	fn from(value: i32) -> Self {
		match value {
			0 => SortOptions::BestMatch,
			1 => SortOptions::Alphabet,
			2 => SortOptions::Popularity,
			3 => SortOptions::Subscribers,
			4 => SortOptions::RecentlyAdded,
			5 => SortOptions::LatestUpdates,
			_ => SortOptions::BestMatch,
		}
	}
}

impl From<SortOptions> for &str {
	fn from(val: SortOptions) -> Self {
		match val {
			SortOptions::BestMatch => "Best Match",
			SortOptions::Alphabet => "Alphabet",
			SortOptions::Popularity => "Popularity",
			SortOptions::Subscribers => "Subscribers",
			SortOptions::RecentlyAdded => "Recently Added",
			SortOptions::LatestUpdates => "Latest Updates",
		}
	}
}
