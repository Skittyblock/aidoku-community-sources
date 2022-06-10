use aidoku::{
	std::{current_date, defaults::defaults_get, html::Node, String, StringRef, Vec},
	MangaContentRating, MangaViewer,
};
use alloc::string::ToString;

pub fn extract_f32_from_string(title: String, text: String) -> f32 {
	text.replace(&title, "")
		.chars()
		.filter(|a| (*a >= '0' && *a <= '9') || *a == ' ' || *a == '.')
		.collect::<String>()
		.split(' ')
		.collect::<Vec<&str>>()
		.into_iter()
		.map(|a| a.parse::<f32>().unwrap_or(0.0))
		.find(|a| *a > 0.0)
		.unwrap_or(0.0)
}

pub fn urlencode(string: String) -> String {
	let mut result: Vec<u8> = Vec::with_capacity(string.len() * 3);
	let hex = "0123456789ABCDEF".as_bytes();
	let bytes = string.as_bytes();

	for byte in bytes {
		let curr = *byte;
		if (b'a'..=b'z').contains(&curr)
			|| (b'A'..=b'Z').contains(&curr)
			|| (b'0'..=b'9').contains(&curr)
		{
			result.push(curr);
		} else {
			result.push(b'%');
			result.push(hex[curr as usize >> 4]);
			result.push(hex[curr as usize & 15]);
		}
	}
	String::from_utf8(result).unwrap_or_default()
}

pub fn get_lang_code() -> String {
	let mut code = String::from("vn");
	if let Ok(languages) = defaults_get("languages").as_array() {
		if let Ok(language) = languages.get(0).as_string() {
			code = language.read();
		}
	}
	code
}

pub fn text_with_newlines(node: Node) -> String {
	let html = node.html().read();
	if !String::from(html.trim()).is_empty() {
		Node::new_fragment(
			node.html()
				.read()
				.replace("<br>", "{{ .LINEBREAK }}")
				.as_bytes(),
		)
		.text()
		.read()
		.replace("{{ .LINEBREAK }}", "\n")
	} else {
		String::new()
	}
}

pub fn category_parser(categories: &Vec<String>) -> (MangaContentRating, MangaViewer) {
	let mut nsfw = MangaContentRating::Safe;
	let mut viewer = MangaViewer::Rtl;
	for category in categories {
		match category.as_str() {
			"Adult" | "Smut" | "Mature" | "18+" => nsfw = MangaContentRating::Nsfw,
			"Ecchi" | "16+" => {
				nsfw = match nsfw {
					MangaContentRating::Nsfw => MangaContentRating::Nsfw,
					_ => MangaContentRating::Suggestive,
				}
			}
			"Webtoon" | "Manhwa" | "Manhua" => viewer = MangaViewer::Scroll,
			"VnComic" => viewer = MangaViewer::Ltr,
			_ => continue,
		}
	}
	(nsfw, viewer)
}

pub fn capitalize_first_letter(name: String) -> String {
	let preprocess = name.chars().collect::<Vec<_>>();
	let mut ret = String::with_capacity(preprocess.len() * 2);
	ret.push_str(&preprocess[0].to_uppercase().to_string());
	let mut i: usize = 1;
	while i < preprocess.len() {
		if preprocess[i].is_whitespace() {
			ret.push(preprocess[i]);
			ret.push_str(&preprocess[i + 1].to_uppercase().to_string());
			i += 1;
		} else {
			ret.push(preprocess[i]);
		}
		i += 1;
	}
	ret
}

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
