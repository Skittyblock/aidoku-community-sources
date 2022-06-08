use aidoku::std::{current_date, String, StringRef, Vec};

use crate::helper::{extract_f32_from_string, get_lang_code, urlencode};

macro_rules! parse_manga_list {
	($elems:expr) => {{
		use aidoku::{
			prelude::format,
			std::{String, Vec},
			Manga, MangaContentRating, MangaStatus, MangaViewer,
		};
        use crate::helper::capitalize_first_letter;
		let mut manga: Vec<Manga> = Vec::with_capacity($elems.len());
		let has_more = $elems.len() > 0;
		for elem in $elems {
			let node = elem.as_node();
			let id = node.select("div.mdl-card__title a").attr("href").read();
			let cover = node
				.select("div.container-3-4.background-contain img")
				.attr("src")
				.read()
                .replace("http:", "https:");
			let title = capitalize_first_letter(
                node
			    	.select("div.mdl-card__supporting-text a[target=_blank]")
				    .text()
				    .read()
            );
			let comic_variant_node = node.select("div.mdl-card__supporting-text a:matchesOwn(Manga|Manhwa|Manhua|.*Novel)");
			let viewer = match comic_variant_node.text().read().trim() {
				"Manhua" | "Manhwa" => MangaViewer::Scroll,
				"Manga" => MangaViewer::Rtl,
				"Light Novel" | "Web Novel" => continue,
                _ => continue,
			};

			manga.push(Manga {
				id: id.clone(),
				cover,
				title: String::from(title.trim()),
				author: String::new(),
				artist: String::new(),
				description: String::new(),
				url: format!("https://otakusan.net{id}"),
				categories: Vec::new(),
				status: MangaStatus::Unknown,
				nsfw: MangaContentRating::Safe,
				viewer,
			})
		}
		(manga, has_more)
	}};
}
pub(crate) use parse_manga_list;

pub fn convert_time(ago: String) -> f64 {
	if ago.contains("cách đây") {
		let multiplier = match ago.split(char::is_whitespace).collect::<Vec<_>>()[3] {
			"giây" => 1.0,
			"phút" => 60.0,
			"tiếng" => 3600.0,
			"ngày" => 86400.0,
			_ => return -1.0,
		};
		current_date() - (extract_f32_from_string(String::new(), ago) * multiplier) as f64
	} else {
		StringRef::from(ago)
			.0
			.as_date("dd/MM/yy", Some("en_US"), Some("Asia/Ho_Chi_Minh"))
			.unwrap_or(-1.0)
	}
}

pub fn url_replacer(url: String, vi: String) -> String {
	let mut url = url;
	url = url
		.replace("_h_", "http")
		.replace("_e_", "/extendContent/Manga")
		.replace("_r_", "/extendContent/MangaRaw");
	if &url[0..2] == "//" {
		url = String::from("https:") + &url;
	}
	if url.contains("drive.google.com") {
		return url;
	}
	url = match &url[0..5] {
		"[GDP]" => url.replace("[GDP]", "https://drive.google.com/uc?export=view&id="),
		"[GDT]" => {
			if &get_lang_code() == "us" {
				url.replace("image2.otakuscan.net", "image3.shopotaku.net")
					.replace("image2.otakusan.net", "image3.shopotaku.net")
			} else {
				url
			}
		}
		"[IS1]" => {
			let mut url_temp = url.replace("[IS1]", "https://imagepi.otakuscan.net/");
			if url_temp.contains("vi=") && !url_temp.contains("otakusan.net_") {
				url_temp
			} else {
				if url.contains('?') {
					url_temp.push_str("&vi=");
				} else {
					url_temp.push_str("?vi=");
				}
				url_temp.push_str(&vi);
				url_temp
			}
		}
		"[IS3]" => url.replace("[IS3]", "https://image3.otakusan.net/"),
		"[IO3]" => url.replace("[IO3]", "http://image3.shopotaku.net/"),
		_ => url,
	};
	if url.contains("/Content/Workshop") || url.contains("otakusan") || url.contains("myrockmanga")
	{
		return url;
	}
	if url.contains("i.blogtruyen") {
		url = url.replace("i.blogtruyen", "i2.blogtruyen");
	}
	if url.contains("file-bato-orig.anyacg.co") {
		url = url.replace("file-bato-orig.anyacg.co", "file-bato-orig.bato.to");
	}
	if url.contains("file-comic") {
		if url.contains("file-comic-1") {
			url = url.replace("file-comic-1.anyacg.co", "z-img-01.mangapark.net");
		}
		if url.contains("file-comic-2") {
			url = url.replace("file-comic-2.anyacg.co", "z-img-02.mangapark.net");
		}
		if url.contains("file-comic-3") {
			url = url.replace("file-comic-3.anyacg.co", "z-img-03.mangapark.net");
		}
		if url.contains("file-comic-4") {
			url = url.replace("file-comic-4.anyacg.co", "z-img-04.mangapark.net");
		}
		if url.contains("file-comic-5") {
			url = url.replace("file-comic-5.anyacg.co", "z-img-05.mangapark.net");
		}
		if url.contains("file-comic-6") {
			url = url.replace("file-comic-6.anyacg.co", "z-img-06.mangapark.net");
		}
		if url.contains("file-comic-9") {
			url = url.replace("file-comic-9.anyacg.co", "z-img-09.mangapark.net");
		}
		if url.contains("file-comic-10") {
			url = url.replace("file-comic-10.anyacg.co", "z-img-10.mangapark.net");
		}
		if url.contains("file-comic-99") {
			url = url.replace("file-comic-99.anyacg.co/uploads", "file-bato-0001.bato.to");
		}
	}
	if url.contains("cdn.nettruyen.com") {
		url = url.replace(
			"cdn.nettruyen.com/Data/Images/",
			"truyen.cloud/data/images/",
		);
	}
	if url.contains("url=") {
		url = String::from(url.split("url=").collect::<Vec<_>>()[1]);
	}
	if url.contains("blogspot") || url.contains("fshare") {
		url = url.replace("http:", "https:")
	}
	if url.contains("blogspot") && !url.contains("http") {
		url = String::from("https://") + &url;
	}
	if url.contains("app/manga/uploads/") && !url.contains("http") {
		url = String::from("https://lhscan.net") + &url;
	}
	url = url.replace("//cdn.adtrue.com/rtb/async.js", "");
	if url.contains(".webp") {
		url = String::from("https://otakusan.net/api/Value/ImageSyncing?ip=34512351&url=")
			+ &urlencode(url);
	} else if (url.contains("merakiscans")
		|| url.contains("mangazuki")
		|| url.contains("ninjascans")
		|| url.contains("anyacg.co")
		|| url.contains("mangakatana")
		|| url.contains("zeroscans")
		|| url.contains("mangapark")
		|| url.contains("mangadex")
		|| url.contains("uptruyen")
		|| url.contains("hocvientruyentranh")
		|| url.contains("ntruyen.info")
		|| url.contains("chancanvas")
		|| url.contains("bato.to"))
		&& (!url.contains("googleusercontent")
			&& !url.contains("otakusan")
			&& !url.contains("otakuscan")
			&& !url.contains("shopotaku"))
	{
		url = String::from("https://images2-focus-opensocial.googleusercontent.com/gadgets/proxy?container=focus&gadget=a&no_expand=1&resize_h=0&rewriteMime=image%2F*&url=") + &urlencode(url);
	} else if url.contains("imageinstant.com") {
		url = String::from("https://images.weserv.nl/?url=") + &urlencode(url);
	} else if !url.contains("otakusan.net") {
		url = String::from("https://otakusan.net/api/Value/ImageSyncing?ip=34512351&url=")
			+ &urlencode(url);
	}
	if url.contains("vi=") && !url.contains("otakusan.net_") {
		url
	} else {
		if url.contains('?') {
			url.push_str("&vi=");
		} else {
			url.push_str("?vi=");
		}
		url.push_str(&vi);
		url
	}
}
