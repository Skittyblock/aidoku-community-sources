#![no_std]
use aidoku::{
	prelude::*, error::Result, std::String, std::Vec, std::ObjectRef, std::net::Request, std::net::HttpMethod,
	Filter, FilterType, Listing, Manga, MangaPageResult, Chapter, Page, DeepLink,
	std::defaults::defaults_get,
};

mod parser;

fn urlencode(string: String) -> String {
	let mut result: Vec<u8> = Vec::with_capacity(string.len() * 3);
	let hex = "0123456789abcdef".as_bytes();
	let bytes = string.as_bytes();
	
	for byte in bytes {
		let curr = *byte;
		if (b'a' <= curr && curr <= b'z')
			|| (b'A' <= curr && curr <= b'Z')
			|| (b'0' <= curr && curr <= b'9') {
				result.push(curr);
		} else {
			result.push(b'%');
			result.push(hex[curr as usize >> 4]);
			result.push(hex[curr as usize & 15]);
		}
	}

	String::from_utf8(result).unwrap_or(String::new())
}

fn i32_to_string(mut integer: i32) -> String {
	if integer == 0 {
		return String::from("0");
	}
	let mut string = String::with_capacity(11);
	let pos = if integer < 0 {
		string.insert(0, '-');
		1
	} else {
		0
	};
	while integer != 0 {
		let mut digit = integer % 10;
		if pos == 1 {
			digit *= -1;
		}
		string.insert(pos, char::from_u32((digit as u32) + ('0' as u32)).unwrap());
		integer /= 10;
	}
	return string;
}

#[get_manga_list]
fn get_manga_list(filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
	let offset = (page - 1) * 20;
	let mut url = String::from("https://api.mangadex.org/manga/?includes[]=cover_art&limit=20&offset=");
	url.push_str(&i32_to_string(offset));

	for filter in filters {
		match filter.kind {
			FilterType::Title => {
				url.push_str("&title=");
				let encoded = urlencode(filter.value.as_string()?.read());
				url.push_str(&encoded);
			},
			FilterType::Author => {
				url.push_str("&author=");
				let encoded = urlencode(filter.value.as_string()?.read());
				url.push_str(&encoded);
			},
			FilterType::Check => {
				if filter.value.as_int().unwrap_or(-1) <= 0 {
					continue;
				}
				match filter.name.as_str() {
					// Original Language
					"Japanese (Manga)" => url.push_str("&originalLanguage[]=ja"),
					"Chinese (Manhua)" => url.push_str("&originalLanguage[]=zh"),
					"Korean (Manhwa)" => url.push_str("&originalLanguage[]=ko"),
					// Demographic
					"None" => url.push_str("&publicationDemographic[]=none"),
					"Shounen" => url.push_str("&publicationDemographic[]=shounen"),
					"Shoujo" => url.push_str("&publicationDemographic[]=shoujo"),
					"Seinen" => url.push_str("&publicationDemographic[]=seinen"),
					"Josei" => url.push_str("&publicationDemographic[]=josei"),
					// Content Rating
					"Safe" => url.push_str("&contentRating[]=safe"),
					"Suggestive" => url.push_str("&contentRating[]=suggestive"),
					"Erotica" => url.push_str("&contentRating[]=erotica"),
					"Pornographic" => url.push_str("&contentRating[]=pornographic"),
					// Status
					"Ongoing" => url.push_str("&status[]=ongoing"),
					"Completed" => url.push_str("&status[]=completed"),
					"Hiatus" => url.push_str("&status[]=hiatus"),
					"Cancelled" => url.push_str("&status[]=cancelled"),
					_ => continue,
				}
			},
			FilterType::Genre => {
				// https://api.mangadex.org/manga/tag
				let tag = match filter.name.as_str() {
					"Action" => "391b0423-d847-456f-aff0-8b0cfc03066b",
					"Adaptation" => "f4122d1c-3b44-44d0-9936-ff7502c39ad3",
					"Adventure" => "87cc87cd-a395-47af-b27a-93258283bbc6",
					"Aliens" => "e64f6742-c834-471d-8d72-dd51fc02b835",
					"Animals" => "3de8c75d-8ee3-48ff-98ee-e20a65c86451",
					"Anthology" => "51d83883-4103-437c-b4b1-731cb73d786c",
					"Award Winning" => "0a39b5a1-b235-4886-a747-1d05d216532d",
					"Boy's Love" => "5920b825-4181-4a17-beeb-9918b0ff7a30",
					"Comedy" => "4d32cc48-9f00-4cca-9b5a-a839f0764984",
					"Cooking" => "ea2bc92d-1c26-4930-9b7c-d5c0dc1b6869",
					"Crime" => "5ca48985-9a9d-4bd8-be29-80dc0303db72",
					"Crossdressing" => "9ab53f92-3eed-4e9b-903a-917c86035ee3",
					"Delinquents" => "da2d50ca-3018-4cc0-ac7a-6b7d472a29ea",
					"Demons" => "39730448-9a5f-48a2-85b0-a70db87b1233",
					"Doujinshi" => "b13b2a48-c720-44a9-9c77-39c9979373fb",
					"Drama" => "b9af3a63-f058-46de-a9a0-e0c13906197a",
					"Fan Colored" => "7b2ce280-79ef-4c09-9b58-12b7c23a9b78",
					"Fantasy" => "cdc58593-87dd-415e-bbc0-2ec27bf404cc",
					"4-Koma" => "b11fda93-8f1d-4bef-b2ed-8803d3733170",
					"Full Color" => "f5ba408b-0e7a-484d-8d49-4e9125ac96de",
					"Genderswap" => "2bd2e8d0-f146-434a-9b51-fc9ff2c5fe6a",
					"Ghosts" => "3bb26d85-09d5-4d2e-880c-c34b974339e9",
					"Girl's Love" => "a3c67850-4684-404e-9b7f-c69850ee5da6",
					"Gore" => "b29d6a3d-1569-4e7a-8caf-7557bc92cd5d",
					"Gyaru" => "fad12b5e-68ba-460e-b933-9ae8318f5b65",
					"Harem" => "aafb99c1-7f60-43fa-b75f-fc9502ce29c7",
					"Historical" => "33771934-028e-4cb3-8744-691e866a923e",
					"Horror" => "cdad7e68-1419-41dd-bdce-27753074a640",
					"Incest" => "5bd0e105-4481-44ca-b6e7-7544da56b1a3",
					"Isekai" => "ace04997-f6bd-436e-b261-779182193d3d",
					"Loli" => "2d1f5d56-a1e5-4d0d-a961-2193588b08ec",
					"Long Strip" => "3e2b8dae-350e-4ab8-a8ce-016e844b9f0d",
					"Mafia" => "85daba54-a71c-4554-8a28-9901a8b0afad",
					"Magic" => "a1f53773-c69a-4ce5-8cab-fffcd90b1565",
					"Magical Girls" => "81c836c9-914a-4eca-981a-560dad663e73",
					"Martial Arts" => "799c202e-7daa-44eb-9cf7-8a3c0441531e",
					"Mecha" => "50880a9d-5440-4732-9afb-8f457127e836",
					"Medical" => "c8cbe35b-1b2b-4a3f-9c37-db84c4514856",
					"Military" => "ac72833b-c4e9-4878-b9db-6c8a4a99444a",
					"Monster Girls" => "dd1f77c5-dea9-4e2b-97ae-224af09caf99",
					"Monsters" => "36fd93ea-e8b8-445e-b836-358f02b3d33d",
					"Music" => "f42fbf9e-188a-447b-9fdc-f19dc1e4d685",
					"Mystery" => "ee968100-4191-4968-93d3-f82d72be7e46",
					"Ninja" => "489dd859-9b61-4c37-af75-5b18e88daafc",
					"Office Workers" => "92d6d951-ca5e-429c-ac78-451071cbf064",
					"Official Colored" => "320831a8-4026-470b-94f6-8353740e6f04",
					"Oneshot" => "0234a31e-a729-4e28-9d6a-3f87c4966b9e",
					"Philosophical" => "b1e97889-25b4-4258-b28b-cd7f4d28ea9b",
					"Police" => "df33b754-73a3-4c54-80e6-1a74a8058539",
					"Post-Apocalyptic" => "9467335a-1b83-4497-9231-765337a00b96",
					"Psychological" => "3b60b75c-a2d7-4860-ab56-05f391bb889c",
					"Reincarnation" => "0bc90acb-ccc1-44ca-a34a-b9f3a73259d0",
					"Reverse Harem" => "65761a2a-415e-47f3-bef2-a9dababba7a6",
					"Romance" => "423e2eae-a7a2-4a8b-ac03-a8351462d71d",
					"Samurai" => "81183756-1453-4c81-aa9e-f6e1b63be016",
					"School Life" => "caaa44eb-cd40-4177-b930-79d3ef2afe87",
					"Sci-Fi" => "256c8bd9-4904-4360-bf4f-508a76d67183",
					"Sexual Violence" => "97893a4c-12af-4dac-b6be-0dffb353568e",
					"Shota" => "ddefd648-5140-4e5f-ba18-4eca4071d19b",
					"Slice of Life" => "e5301a23-ebd9-49dd-a0cb-2add944c7fe9",
					"Sports" => "69964a64-2f90-4d33-beeb-f3ed2875eb4c",
					"Superhero" => "7064a261-a137-4d3a-8848-2d385de3a99c",
					"Supernatural" => "eabc5b4c-6aff-42f3-b657-3e90cbd00b75",
					"Survival" => "5fff9cde-849c-4d78-aab0-0d52b2ee1d25",
					"Thriller" => "07251805-a27e-4d59-b488-f0bfbec15168",
					"Time Travel" => "292e862b-2d17-4062-90a2-0356caa4ae27",
					"Tragedy" => "f8f62932-27da-4fe4-8ee1-6779a8c5edba",
					"Traditional Games" => "31932a7e-5b8e-49a6-9f12-2afa39dc544c",
					"User Created" => "891cf039-b895-47f0-9229-bef4c96eccd4",
					"Vampires" => "d7d1730f-6eb0-4ba6-9437-602cac38664c",
					"Video Games" => "9438db5a-7e2a-4ac0-b39e-e0d95a34b8a8",
					"Villainess" => "d14322ac-4d6f-4e9b-afd9-629d5f4d8a41",
					"Virtual Reality" => "8c86611e-fab7-4986-9dec-d1a2f44acdd5",
					"Web Comic" => "e197df38-d0e7-43b5-9b09-2842d0c326dd",
					"Wuxia" => "acc803a4-c95a-4c22-86fc-eb6b582d82a2",
					"Zombies" => "631ef465-9aba-4afb-b0fc-ea10efe274a8",
					_ => continue,
				};
				match filter.value.as_int().unwrap_or(-1) {
					0 => url.push_str("&excludedTags[]="),
					1 => url.push_str("&includedTags[]="),
					_ => continue,
				}
				url.push_str(tag);
			},
			FilterType::Sort => {
				let value = match filter.value.as_object() {
					Ok(value) => value,
					Err(_) => continue,
				};
				let index = value.get("index").as_int().unwrap_or(0);
				let ascending = value.get("ascending").as_bool().unwrap_or(false);
				let option = match index {
					0 => "latestUploadedChapter",
					1 => "relevance",
					2 => "followedCount",
					3 => "createdAt",
					4 => "updatedAt",
					5 => "title",
					_ => continue,
				};
				url.push_str("&order[");
				url.push_str(&option);
				url.push_str("]=");
				url.push_str(&if ascending { "asc" } else { "desc" });
			},
			_ => continue,
		}
	}

	let json = Request::new(&url, HttpMethod::Get).json().as_object()?;

	let data = json.get("data").as_array()?;

	let mut manga_arr: Vec<Manga> = Vec::new();

	for manga in data {
		let manga_obj = manga.as_object()?;
		if let Ok(manga) = parser::parse_basic_manga(manga_obj) {
			manga_arr.push(manga);
		}
	}

	let total = json.get("total").as_int().unwrap_or(0) as i32;

	Ok(MangaPageResult {
		manga: manga_arr,
		has_more: offset + 20 < total,
	})
}

#[get_manga_listing]
fn get_manga_listing(listing: Listing, page: i32) -> Result<MangaPageResult> {
	let mut filters: Vec<Filter> = Vec::new();

	let mut selection = ObjectRef::new();

	if listing.name == "Popular" {
		selection.set("index", 2i32.into());
		selection.set("ascending", false.into());
		filters.push(Filter {
			kind: FilterType::Sort,
			name: String::from("Sort"),
			value: selection.0,
		});
	} else if listing.name == "Latest" { // get recently published chapters
		let offset = (page - 1) * 20;
		let mut url = String::from("https://api.mangadex.org/chapter?includes[]=manga&order[publishAt]=desc&includeFutureUpdates=0&limit=20&offset=");
		url.push_str(&i32_to_string(offset));
		if let Ok(languages) = defaults_get("languages").as_array() {
			for lang in languages {
				if let Ok(lang) = lang.as_string() {
					url.push_str("&translatedLanguage[]=");
					url.push_str(&lang.read());
				}
			}
		}

		let json = Request::new(&url, HttpMethod::Get).json().as_object()?;

		let data = json.get("data").as_array()?;
	
		let mut manga_arr: Vec<Manga> = Vec::new();
		let mut manga_ids: Vec<String> = Vec::new();
	
		for chapter in data {
			if let Ok(chapter_obj) = chapter.as_object() {
				if let Ok(relationships) = chapter_obj.get("relationships").as_array() {
					for relationship in relationships {
						if let Ok(relationship_obj) = relationship.as_object() {
							let relation_type = relationship_obj.get("type").as_string()?.read();
							if relation_type == "manga" {
								let id = relationship_obj.get("id").as_string()?.read();
								if manga_ids.contains(&id) {
									continue;
								}
								if let Ok(parsed_manga) = get_manga_details(id) {
									manga_ids.push(parsed_manga.id.clone());
									manga_arr.push(parsed_manga);
								}
								break;
							}
						}
					}
				}
			}
		}

		let total = json.get("total").as_int().unwrap_or(0) as i32;
	
		return Ok(MangaPageResult {
			manga: manga_arr,
			has_more: offset + 20 < total,
		});
	}

	let result = get_manga_list(filters, page);

	result
}

#[get_manga_details]
fn get_manga_details(id: String) -> Result<Manga> {
	let mut url = String::from("https://api.mangadex.org/manga/");
	url.push_str(&id);
	url.push_str("?includes[]=cover_art&includes[]=author&includes[]=artist");
	let json = Request::new(&url, HttpMethod::Get).json().as_object()?;

	let data = json.get("data").as_object()?;

	parser::parse_full_manga(data)
}

#[get_chapter_list]
fn get_chapter_list(id: String) -> Result<Vec<Chapter>> {
	let mut url = String::from("https://api.mangadex.org/manga/");
	url.push_str(&id);
	url.push_str("/feed?order[volume]=desc&order[chapter]=desc&limit=500&contentRating[]=pornographic&contentRating[]=erotica&contentRating[]=suggestive&contentRating[]=safe&includes[]=scanlation_group");
	if let Ok(languages) = defaults_get("languages").as_array() {
		for lang in languages {
			if let Ok(lang) = lang.as_string() {
				url.push_str("&translatedLanguage[]=");
				url.push_str(&lang.read());
			}
		}
	}
	if let Ok(groups_string) = defaults_get("blockedGroups").as_string() {
		groups_string.read().split(",").for_each(|group| {
			url.push_str("&excludedGroups[]=");
			url.push_str(group);
		});
	}

	let json = Request::new(&url, HttpMethod::Get).json().as_object()?;

	let data = json.get("data").as_array()?;

	let mut chapters: Vec<Chapter> = Vec::new();

	for chapter in data {
		if let Ok(chapter_obj) = chapter.as_object() {
			if let Ok(chapter) = parser::parse_chapter(chapter_obj) {
				chapters.push(chapter);
			}
		}
	}

	Ok(chapters)
}

#[get_page_list]
fn get_page_list(id: String) -> Result<Vec<Page>> {
	let mut url = String::from("https://api.mangadex.org/at-home/server/");
	url.push_str(&id);
	let json = Request::new(&url, HttpMethod::Get).json().as_object()?;

	let chapter = json.get("chapter").as_object()?;
	let data = chapter.get(if defaults_get("dataSaver").as_bool().unwrap_or(false) { "dataSaver" } else { "data" }).as_array()?;

	let base_url = json.get("baseUrl").as_string()?.read();
	let hash = chapter.get("hash").as_string()?.read();

	let mut pages: Vec<Page> = Vec::new();

	let mut i = 0;
	for page in data {
		let data_string = page.as_string()?.read();
		let mut url = String::new();
		url.push_str(&base_url);
		if defaults_get("dataSaver").as_bool().unwrap_or(false) {
			url.push_str("/data-saver/");
		} else {
			url.push_str("/data/");
		}
		url.push_str(&hash);
		url.push_str("/");
		url.push_str(&data_string);

		pages.push(Page {
			index: i,
			url,
			base64: String::new(),
			text: String::new(),
		});

		i += 1;
	}

	Ok(pages)
}

#[handle_url]
pub fn handle_url(url: String) -> Result<DeepLink> {
	let url = &url[21..]; // remove "https://mangadex.org/"

	if url.starts_with("title") { // ex: https://mangadex.org/title/a96676e5-8ae2-425e-b549-7f15dd34a6d8/komi-san-wa-komyushou-desu
		let id = &url[6..]; // remove "title/"
		let end = match id.find("/") {
			Some(end) => end,
			None => id.len(),
		};
		let manga_id = &id[..end];
		let manga = get_manga_details(String::from(manga_id))?;

		return Ok(DeepLink {
			manga: Some(manga),
			chapter: None,
		});
	} else if url.starts_with("chapter") { // ex: https://mangadex.org/chapter/56eecc6f-1a4e-464c-b6a4-a1cbdfdfd726/1
		let id = &url[8..]; // remove "chapter/"
		let end = match id.find("/") {
			Some(end) => end,
			None => id.len(),
		};
		let chapter_id = &id[..end];

		let mut url = String::from("https://api.mangadex.org/chapter/");
		url.push_str(&chapter_id);

		let json = Request::new(&url, HttpMethod::Get).json().as_object()?;

		let chapter_obj = json.get("data").as_object()?;
		let relationships = chapter_obj.get("relationships").as_array()?;
		for relationship in relationships {
			if let Ok(relationship_obj) = relationship.as_object() {
				let relation_type = relationship_obj.get("type").as_string()?.read();
				if relation_type == "manga" {
					let manga_id = relationship_obj.get("id").as_string()?.read();
					let manga = get_manga_details(String::from(manga_id))?;
					let chapter = parser::parse_chapter(chapter_obj)?;
					return Ok(DeepLink {
						manga: Some(manga),
						chapter: Some(chapter),
					});
				}
			}
		}
	}

	Err(aidoku::error::AidokuError { reason: aidoku::error::AidokuErrorKind::Unimplemented })
}
