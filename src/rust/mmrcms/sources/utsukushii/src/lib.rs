#![no_std]
use mmrcms_template::{mmrcms, template::MMRCMSSource};

mmrcms! {
	MMRCMSSource {
		base_url: "https://manga.utsukushii-bg.com",
		lang: "bg",
		category: "Жанр",
		detail_categories: "Жанр",
		detail_description: "Резюме",
		category_mapper: |idx| {
			match idx {
				0 => String::new(),
				1..=3 => format!("{}", idx),
				4 => format!("{}", idx + 1),
				5 => format!("{}", idx + 2),
				6..=9 => format!("{}", idx + 4),
				10..=18 => format!("{}", idx + 5),
				19..=22 => format!("{}", idx + 8),
				23..=27 => format!("{}", idx + 11),
				_ => String::new(),
			}
		},
		..Default::default()
	}
}
