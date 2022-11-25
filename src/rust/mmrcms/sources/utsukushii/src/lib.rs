#![no_std]
use mmrcms_template::{mmrcms, template::MMRCMSSource};

mmrcms! {
	MMRCMSSource {
		base_url: "https://manga.utsukushii-bg.com",
		lang: "bg",
		category: "Жанр",
		category_mapper: |idx| {
			match idx {
				0 => String::new(),
				1..=3 => String::from(itoa::Buffer::new().format(idx)),
				4 => String::from(itoa::Buffer::new().format(idx + 1)),
				5 => String::from(itoa::Buffer::new().format(idx + 2)),
				6..=9 => String::from(itoa::Buffer::new().format(idx + 4)),
				10..=18 => String::from(itoa::Buffer::new().format(idx + 5)),
				19..=22 => String::from(itoa::Buffer::new().format(idx + 8)),
				23..=27 => String::from(itoa::Buffer::new().format(idx + 11)),
				_ => String::new(),
			}
		},
		..Default::default()
	}
}
