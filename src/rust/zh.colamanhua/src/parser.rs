use aidoku::{
	error::Result,
	std::{
		html::Node,
		net::{HttpMethod, Request},
		String, Vec,
	},
	Chapter, Filter, Manga, MangaPageResult, MangaViewer, Page,
};

const BASE_URL: &str = "https://www.colamanhua.com";

const FILTER_GENRE: [&str; 31] = [
	"", "10023", "10024", "10126", "10124", "10210", "10143", "10129", "10242", "10560", "10122",
	"10641", "10201", "10138", "10461", "10943", "10301", "10321", "10309", "10125", "10131",
	"10133", "10127", "10142", "10722", "10480", "10706", "11062", "10227", "10183", "10181",
];
const FILTER_STATUS: [&str; 3] = ["", "1", "2"];
const FILTER_ALPHABET: [&str; 27] = [
	"", "10182", "10081", "10134", "10001", "10238", "10161", "10225", "10137", "10284", "10141",
	"10283", "10132", "10136", "10130", "10282", "10262", "10164", "10240", "10121", "10123",
	"11184", "11483", "10135", "10061", "10082", "10128",
];

const SORT: [&str; 4] = ["update", "dailyCount", "weeklyCount", "monthlyCount"];

pub fn get_filtered_url(filters: Vec<Filter>, page: i32, url: &mut String) {
	todo!();
}

pub fn request_get(url: &mut String) -> Request {
	Request::new(url.as_str(), HttpMethod::Get).header("User-Agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 13_3_1) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/112.0.0.0 Safari/537.36")
}

pub fn parse_home_page(html: Node, page: i32) -> Result<MangaPageResult> {
	let mut mangas: Vec<Manga> = Vec::new();
	let mut has_more = false;

	todo!();

	Ok(MangaPageResult {
		manga: mangas,
		has_more,
	})
}

pub fn parse_search_page(html: Node, page: i32) -> Result<MangaPageResult> {
	let mut mangas: Vec<Manga> = Vec::new();
	let mut has_more = false;

	todo!();

	Ok(MangaPageResult {
		manga: mangas,
		has_more,
	})
}

pub fn get_manga_details(html: Node, manga_id: String) -> Result<Manga> {
	todo!();

	Ok(Manga {
		id: todo!(),
		cover: todo!(),
		title: todo!(),
		author: todo!(),
		artist: todo!(),
		description: todo!(),
		url: todo!(),
		categories: todo!(),
		status: todo!(),
		viewer: MangaViewer::Scroll,
		..Default::default()
	})
}

pub fn get_chapter_list(html: Node, manga_id: String) -> Result<Vec<Chapter>> {
	let mut chapters: Vec<Chapter> = Vec::new();

	todo!();

	Ok(chapters)
}

pub fn get_page_list(html: Node) -> Result<Vec<Page>> {
	let mut pages: Vec<Page> = Vec::new();

	todo!();

	Ok(pages)
}

pub fn parse_deep_link(deep_link: &mut String) -> (Option<String>, Option<String>) {
	let mut manga_id = None;
	let mut chapter_id = None;

	todo!();

	(manga_id, chapter_id)
}
