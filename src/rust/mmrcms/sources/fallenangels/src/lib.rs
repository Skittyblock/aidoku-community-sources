#![no_std]
use mmrcms_template::{mmrcms, template::MMRCMSSource};

mmrcms! {
	MMRCMSSource {
		base_url: "https://manga.fascans.com",
		tags_mapper: |idx| {
			String::from(match idx {
				1 => "7", // One-shot
				_ => "",
			})
		},
		..Default::default()
	}
}
