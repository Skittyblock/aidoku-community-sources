#![no_std]
use aidoku::prelude::format;
use mmrcms_template::{mmrcms, template::MMRCMSSource};

mmrcms! {
	MMRCMSSource {
		base_url: "https://www.komikid.com",
		category_mapper: |idx| {
			match idx {
				0 => String::new(),
				1..=5 => format!("{}", idx),
				6..=31 => format!("{}", idx + 1),
				_ => String::new(),
			}
		},
		..Default::default()
	}
}
