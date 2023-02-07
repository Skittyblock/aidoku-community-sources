#![no_std]
use aidoku::{MangaContentRating, MangaViewer};
use mmrcms_template::{mmrcms, template::MMRCMSSource};

mmrcms! {
	MMRCMSSource {
		base_url: "https://raws.mangazuki.co",
		lang: "ko",
		// While Mangazuki does have some mangas, there's really no way to discern
		// them from manwahs, which take up most of the content.
		// For the same reason, everything is NSFW, despite that not actually
		// being the case.
		category_parser: |_, _| (MangaContentRating::Nsfw, MangaViewer::Scroll),
		..Default::default()
	}
}
