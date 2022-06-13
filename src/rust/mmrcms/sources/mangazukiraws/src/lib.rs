#![no_std]
use mmrcms_template::{mmrcms, template::MMRCMSSource};

mmrcms! {
	MMRCMSSource {
		base_url: "https://raws.mangazuki.co",
		lang: "ko",
		..Default::default()
	}
}
