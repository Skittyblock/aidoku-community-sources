#![no_std]
use aidoku::{MangaContentRating, MangaViewer};
use mmrcms_template::{mmrcms, template::MMRCMSSource};

mmrcms! {
	MMRCMSSource {
		base_url: "https://hentai.gekkouscans.com.br",
		lang: "pt-BR",
		category: "Categoria",
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
