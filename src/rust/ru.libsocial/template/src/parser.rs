use aidoku::error::{AidokuError, AidokuErrorKind};
use aidoku::std::defaults::defaults_get;
use aidoku::std::html::Node;
use aidoku::std::json::parse;
use aidoku::std::{print, ArrayRef, ObjectRef, StringRef, ValueRef};
use aidoku::{
	error::Result,
	helpers::uri::encode_uri,
	prelude::*,
	std::{String, Vec},
	Chapter, Manga, MangaContentRating, MangaPageResult, MangaStatus, MangaViewer, Page,
};

pub fn manga_list_parse<T: AsRef<str>>(
	base_url: T,
	response: ObjectRef,
) -> Result<MangaPageResult> {
	let items = response.get("items").as_object()?;
	let data = items.get("data").as_array()?;
	let manga = data
		.filter_map(move |manga| match manga.as_object() {
			Ok(obj) => manga_parse(&base_url, obj).ok(),
			Err(_) => None,
		})
		.collect::<Vec<Manga>>();

	let has_more = items.get("next_page_url").is_some();

	Ok(MangaPageResult { manga, has_more })
}

pub fn manga_parse<T: AsRef<str>>(base_url: T, response: ObjectRef) -> Result<Manga> {
	let title = response.get("rus_name").as_string()?.read();
	let cover = response.get("coverImage").as_string()?.read();
	let slug = response.get("slug").as_string()?.read();
	Ok(Manga {
		id: format!("{}", slug),
		cover,
		title,
		author: String::from(""),
		artist: String::from(""),
		description: String::from(""),
		url: format!("{}/{}", base_url.as_ref(), slug),
		categories: Vec::new(),
		status: Default::default(),
		nsfw: Default::default(),
		viewer: Default::default(),
	})
}

pub fn manga_details_parse<T: AsRef<str>>(base_url: T, id: &str, node: Node) -> Result<Manga> {
	let scripts = node.data().read();
	let data_str = scripts
		.split_once("window.__DATA__ = ")
		.and_then(|(_, d)| d.split_once("window._SITE_COLOR_"))
		.and_then(|(d, _)| d.rsplit_once(';'))
		.map(|(d, _)| d)
		.unwrap_or("");
	let data = parse(data_str.as_bytes())?.as_object()?;
	let manga = data.get("manga").as_object()?;

	let category = node
		.select("div.media-info-list__title:contains(Тип) + div")
		.text()
		.read();

	let age = node
		.select("div.media-info-list__title:contains(Возрастной рейтинг) + div")
		.text()
		.read();

	let rating = node
		.select(".media-rating__value")
		.last()
		.text()
		.read()
		.parse::<f64>()
		.unwrap_or(0f64)
		* 2f64;

	let rating_star = if rating > 9.5 {
		"★★★★★"
	} else if rating > 8.5 {
		"★★★★✬"
	} else if rating > 7.5 {
		"★★★★☆"
	} else if rating > 6.5 {
		"★★★✬☆"
	} else if rating > 5.5 {
		"★★★☆☆"
	} else if rating > 4.5 {
		"★★✬☆☆"
	} else if rating > 3.5 {
		"★★☆☆☆"
	} else if rating > 2.5 {
		"★✬☆☆☆"
	} else if rating > 1.5 {
		"★☆☆☆☆"
	} else if rating > 0.5 {
		"✬☆☆☆☆"
	} else {
		"☆☆☆☆☆"
	};
	let rating_votes = node.select(".media-rating__votes").last().text().read();

	let mut genres = Vec::new();
	let tags = node.select(".media-tags > a");
	for tags_node in tags.array() {
		let node = tags_node.as_node().expect("value is not node");
		let genre = node.text().read();

		genres.push(genre);
	}

	let title = if manga.get("engName").is_some() {
		manga.get("engName").as_string()?.read()
	} else {
		manga.get("name").as_string()?.read()
	};

	let author =
		parse_media_info_list(&node, "div.media-info-list__title:contains(Автор) + div a")?;

	let artist = parse_media_info_list(
		&node,
		"div.media-info-list__title:contains(Художник) + div a",
	)?;
	let status_translate = node
		.select("div.media-info-list__title:contains(Статус перевода) + div")
		.text()
		.read()
		.to_lowercase();

	let status_title = node
		.select("div.media-info-list__title:contains(Статус тайтла) + div")
		.text()
		.read()
		.to_lowercase();

	let status = if status_translate.contains("завершен") && status_title.contains("приостановлен")
		|| status_translate.contains("заморожен")
		|| status_translate.contains("заброшен")
	{
		MangaStatus::Hiatus
	} else if status_translate.contains("завершен") && status_title.contains("выпуск прекращён")
	{
		MangaStatus::Cancelled
	} else if status_translate.contains("продолжается") {
		MangaStatus::Ongoing
	} else if status_translate.contains("завершен") {
		MangaStatus::Completed
	} else {
		match status_title.as_str() {
			"онгоинг" | "анонс" => MangaStatus::Ongoing,
			"завершён" => MangaStatus::Completed,
			"приостановлен" => MangaStatus::Hiatus,
			"выпуск прекращён" => MangaStatus::Cancelled,
			_ => MangaStatus::Cancelled,
		}
	};

	let cover = node.select(".media-header__cover").attr("src").read();

	let categories = {
		let size = 2 + genres.len();
		let mut vec = Vec::with_capacity(size);
		vec.push(category.clone());
		if !age.is_empty() {
			vec.push(age.clone());
		}
		vec.extend(genres);
		vec
	};

	let alt_names = String::from("");
	/*	{ JOIN NOT WORKING WITH SOME CHARACTERS
		let mut names = Vec::new();
		 for name in manga
			.get("altNames")
			.as_array()? {
			names.push(name.as_string()?.read());
		}
		if !names.is_empty() {
			format!("Альтернативные названия:\n {} \n\n", names.join(", "))
		} else {
			String::from("")
		}
	};*/

	let media_title = if manga.get("rusName").is_some() {
		manga.get("rusName").as_string()?.read()
	} else {
		String::from("")
	};
	let description = node.select(".media-description__text").text().read();
	let nsfw = match age.as_ref() {
		"18+" => MangaContentRating::Nsfw,
		"16+" => MangaContentRating::Suggestive,
		_ => MangaContentRating::Safe,
	};
	let viewer = match category.as_ref() {
		"Манхва" | "Маньхуа" => MangaViewer::Scroll,
		_ => MangaViewer::default(),
	};
	Ok(Manga {
		id: format!("{}", id),
		url: format!("{}/{}", base_url.as_ref(), id),
		cover,
		title,
		author,
		artist,
		description: format!(
			"{}\n{} {}(голосов: {})\n{}{}",
			media_title, rating_star, rating, rating_votes, alt_names, description
		),
		categories,
		status,
		nsfw,
		viewer,
	})
}
pub fn chapters_parse<T: AsRef<str>>(base_url: T, id: &str, node: Node) -> Result<Vec<Chapter>> {
	let scripts = node.data().read();
	let data_str = scripts
		.split_once("window.__DATA__ = ")
		.and_then(|(_, d)| d.split_once("window._SITE_COLOR_"))
		.and_then(|(d, _)| d.rsplit_once(';'))
		.map(|(d, _)| d)
		.unwrap_or("");

	let data = parse(data_str.as_bytes())?.as_object()?;
	let chapters_info = data.get("chapters").as_object()?;
	let chapters = chapters_info.get("list").as_array()?;

	let auth = data.get("auth").as_bool()?;

	let user_id = if auth {
		data.get("user").as_object()?.get("id").as_string()?.read()
	} else {
		String::from("not")
	};

	let branches = chapters_info.get("branches").as_array()?;

	let teams = chapters_info.get("teams").as_array()?;

	let parsed_chapters = if !branches.is_empty() {
		let mut chs = Vec::new();
		for branch in branches.clone() {
			let branch_obj = branch.as_object()?;
			let team_id = branch_obj.get("id").as_int()?;
			let translate_type = defaults_get("translate_type")?.as_string()?.read();
			for ch in chapters.clone() {
				let chapter_obj = ch.as_object()?;
				if chapter_obj.get("branch_id").as_int().unwrap_or_default() == team_id
					&& chapter_obj.get("status").as_int().unwrap_or_default() != 2
				{
					let branch = branches.clone();
					chs.push(chapter_parse(
						chapter_obj,
						base_url.as_ref(),
						id,
						&user_id,
						Some(team_id),
						Some(branch),
						None,
					)?)
				}
			}

			if translate_type == "mixing" {
				chs.dedup_by_key(|ch| (ch.volume, ch.chapter))
			}
		}
		chs
	} else {
		let mut chs = Vec::new();
		for ch in chapters {
			let chapter_object = ch.as_object()?;
			if chapter_object.get("status").as_int().unwrap_or_default() != 2 {
				chs.push(chapter_parse(
					chapter_object,
					base_url.as_ref(),
					id,
					&user_id,
					None,
					None,
					Some(teams.clone()),
				)?)
			}
		}
		chs
	};
	Ok(parsed_chapters)
}
pub fn chapter_parse(
	chapter: ObjectRef,
	base_url: &str,
	slug: &str,
	user_id: &str,
	team_id: Option<i64>,
	branches: Option<ArrayRef>,
	teams: Option<ArrayRef>,
) -> Result<Chapter> {
	let volume = chapter.get("chapter_volume").as_float()? as f32;

	let number = chapter.get("chapter_number").as_float()? as f32;
	let team_id = if let Some(param) = team_id {
		format!("&bid={}", param)
	} else {
		format!("")
	};
	let scanlator_id = chapter.get("chapter_scanlator_id").as_int()?;
	let is_scanlator_id = if let Some(available_teams) = teams.as_ref() {
		is_scanlator_id(available_teams.clone(), scanlator_id)?
	} else {
		Vec::new()
	};

	let scanlator = if let Some(available_teams) = teams {
		if available_teams.len() == 1 {
			available_teams
				.get(0)
				.as_object()?
				.get("name")
				.as_string()?
				.read()
		} else {
			String::new()
		}
	} else if let Some(scanlator_id) = is_scanlator_id.get(0) {
		scanlator_id.get("name").as_string()?.read()
	} else if let Some(branches) = branches {
		if let Some(team) = scanlator_team_parse(branches, &chapter)? {
			team
		} else {
			String::new()
		}
	} else {
		String::new()
	};
	let title = chapter
		.get("chapter_name")
		.as_string()
		.unwrap_or(StringRef::default())
		.read();
	let date = chapter
		.get("chapter_created_at")
		.as_date("yyyy-MM-dd HH:mm:ss", None, None)?;
	Ok(Chapter {
		id: format!(
			"v{}/c{}?ui={}{}",
			volume,
			(100.0 * number) / 100.0,
			user_id,
			team_id
		),
		title,
		volume,
		chapter: number,
		date_updated: date,
		scanlator,
		url: format!(
			"{}/{}/v{}/c{}?ui={}{}",
			base_url,
			slug,
			volume,
			(100.0 * number) / 100.0,
			user_id,
			team_id
		),
		lang: String::from("ru"),
	})
}

fn scanlator_team_parse(branches: ArrayRef, chapter: &ObjectRef) -> Result<Option<String>> {
	let mut scanlator = None;
	for branch_val in branches {
		let branch = branch_val.as_object()?;

		let teams = branch.get("teams").as_array()?;

		if chapter.get("branch_id").as_int()? == branch.get("id").as_int()? && !teams.is_empty() {
			for team_val in teams.clone() {
				let team = team_val.as_object()?;
				let scanlator_id = chapter.get("chapter_scanlator_id").as_int()?;
				if (scanlator_id == team.get("id").as_int()?)
					|| (scanlator_id == 0 && team.get("is_active").as_int()? == 1)
				{
					return Ok(Some(team.get("name").as_string()?.read()));
				} else {
					let first_team = teams.get(0).as_object()?;

					scanlator = Some(first_team.get("name").as_string()?.read());
				}
			}
		}
	}
	Ok(scanlator)
}

fn is_scanlator_id(teams: ArrayRef, id: i64) -> Result<Vec<ObjectRef>> {
	let mut vec = Vec::new();
	for team in teams {
		let team_object = team.as_object()?;
		if team_object.get("id").as_int() == Ok(id) {
			vec.push(team_object);
		}
	}

	Ok(vec)
}

pub fn parse_media_info_list(node: &Node, query: &str) -> Result<String> {
	let mut infos = Vec::new();
	for list_node in node.select(query).array() {
		infos.push(list_node.as_node()?.text().read());
	}
	Ok(infos.join(", "))
}

pub fn pages_parse(node: Node) -> Result<Vec<Page>> {
	let redirect = node.html().read();

	if !redirect.contains("window.__info") {
		// TODO return error
		return Err(AidokuError {
			reason: AidokuErrorKind::DefaultNotFound,
		});
	}
	let scripts = node.data().read();

	let chapter_info = scripts
		.split("window.__info = ")
		.last()
		.unwrap()
		.trim()
		.split(';')
		.next()
		.unwrap();

	let info_object = parse(chapter_info)?.as_object()?;

	let servers = info_object.get("servers").as_object()?;
	let img = info_object.get("img").as_object()?;
	let default_server = img.get("server").as_string()?.read();
	let selected_server = defaults_get("server").unwrap().as_string()?.read();

	let img_url = img.get("url").as_string()?.read();

	let server = match selected_server.as_ref() {
		"auto" => servers.get(&default_server).as_string()?.read(),
		s => servers.get(s).as_string()?.read(),
	};

	let pages_str = scripts
		.split("window.__pg = ")
		.last()
		.unwrap()
		.trim()
		.split(';')
		.next()
		.unwrap();

	let pages_array = parse(pages_str)?.as_array()?;

	let mut pages = Vec::new();

	for page in pages_array {
		let page_obj = page.as_object()?;

		let url = encode_uri(format!(
			"{}{}{}",
			server,
			img_url,
			page_obj.get("u").as_string()?.read()
		));

		pages.push(Page {
			index: page_obj.get("p").as_int()? as i32,
			url,
			base64: String::from(""),
			text: String::from(""),
		})
	}

	Ok(pages)
}
