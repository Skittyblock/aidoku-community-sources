#![no_std]
use mmrcms_template::{mmrcms, template::MMRCMSSource};

mmrcms! {
	MMRCMSSource {
		base_url: "https://www.komikid.com",
		category_mapper: |idx| {
			match idx {
				0 => String::new(),
				1..=5 => String::from(itoa::Buffer::new().format(idx)),
				6..=31 => String::from(itoa::Buffer::new().format(idx + 1)),
				_ => String::new(),
			}
		},
		..Default::default()
	}
}
