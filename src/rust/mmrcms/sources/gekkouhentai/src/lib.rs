#![no_std]
use mmrcms_template::{mmrcms, template::MMRCMSSource};
use aidoku::{MangaContentRating, MangaViewer};

mmrcms! {
	MMRCMSSource {
		base_url: "https://hentai.gekkouscans.com.br",
		detail_categories: "Categorias",
		category_parser: |_, categories| {
			let mut viewer = MangaViewer::Rtl;
			for category in categories {
				match category.as_str() {
					"Webtoon" => viewer = MangaViewer::Scroll,
					_ => continue,
				}
			}
			(MangaContentRating::Nsfw, viewer)
		},
		category_mapper: |idx| {
			if idx == 0 {
				String::new()
			} else if (1..=7).contains(&idx) {
				format!("{}", idx)
			} else {
				format!("{}", idx + 1)
			}
		},
		..Default::default()
	}
}
