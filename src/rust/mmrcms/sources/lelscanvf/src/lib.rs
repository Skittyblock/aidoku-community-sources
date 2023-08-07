#![no_std]
use mmrcms_template::{mmrcms, template::MMRCMSSource};

extern crate alloc;
use alloc::string::ToString;

mmrcms! {
	MMRCMSSource {
		base_url: "https://lelscanvf.cc",
		lang: "fr",
		category_mapper: |idx| {
			idx.to_string()
		},
		..Default::default()
	}
}
