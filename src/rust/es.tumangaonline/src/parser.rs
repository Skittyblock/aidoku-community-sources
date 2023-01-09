use aidoku::{
	error::AidokuError,
	helpers::substring::Substring,
	std::{net::HttpMethod, net::Request, String, Vec},
	Manga, MangaPageResult,
};
use alloc::{borrow::ToOwned, string::ToString};

use crate::{BASE_URL, USER_AGENT};

pub fn parse_manga_list(url: String) -> Result<MangaPageResult, AidokuError> {
	let html = Request::new(url, HttpMethod::Get)
		.header("User-Agent", USER_AGENT)
		.header("Referer", BASE_URL)
		.html()?;

	let elements = html.select("div.element");

	let mut manga: Vec<Manga> = Vec::new();

	for element in elements.array() {
		let el = element.as_node().expect("html array not an array of nodes");
		let a = el.select("a");
		let item = a.first();
		let url = item.attr("href").read().trim_start().to_owned();
		let id = url.strip_prefix(BASE_URL).unwrap_or(&url).to_owned();
		let title = item.select("h4.text-truncate").text().read();

		let cover = {
			let style = item.select("style").html().read();
			style
				.substring_after("('")
				.unwrap_or_default()
				.substring_before("')")
				.unwrap_or_default()
				.to_string()
		};

		manga.push(Manga {
			id,
			cover,
			title,
			url,
			..Default::default()
		})
	}

	let has_more = !manga.is_empty();

	Ok(MangaPageResult { manga, has_more })
}
