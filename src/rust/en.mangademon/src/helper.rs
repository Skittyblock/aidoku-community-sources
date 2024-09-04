use aidoku::{
	error::Result, prelude::format, std::{Vec,net::{HttpMethod, Request}, String}
};

use crate::BASE_URL;

/// Returns the ID of a manga from a URL.
pub fn get_manga_id(url: &str) -> String {
  // Two Formats
  // https://demonicscans.org/title/Overgeared/chapter/1/2024090306
  // https://demonicscans.org/manga/Overgeared
  // For the chapter format it seems as if the ending part is <year><month><day><hour> where hour is 12hr time in UTC

	let id_with_suffix= url.split("/manga/").last().unwrap_or(url.split("/title/").last().unwrap_or(""));

	// Handle chapter suffix 
	let id_without_chapter_suffix = if let Some(index) = id_with_suffix.rfind("/chapter/") {
		&id_with_suffix[..index]
	} else {
		id_with_suffix
	};

	String::from(id_without_chapter_suffix)
}

/// Returns the ID of a chapter from a URL.
pub fn get_chapter_id(url: &str) -> String {
	// Example Url: https://demonicscans.org/chaptered.php?manga=<num>&chapter=1
	// Example Url: https://demonicscans.org/title/Overgeared/chapter/1/2024090306
	// parse "1" from the url

  if url.contains("chaptered"){
    String::from(url.split("=").last().unwrap_or(""))
  } else{
    let split_url = url.split("/").collect::<Vec<_>>();
    String::from(split_url[split_url.len()-2])
  }
}

/// Returns full URL of a manga from a manga ID.
pub fn get_manga_url(manga_id: &String) -> String {
	format!("{}/manga/{}", BASE_URL, manga_id)
}

pub fn get_chapter_url(manga_id: &String, chapter_id: &String) -> Result<String>{
  let manga_url = get_manga_url(manga_id);
  let html = Request::new(manga_url,HttpMethod::Get).html()?;

  let chap_link = html.select(".chplinks").first();
  let raw_url = chap_link.attr("href").read();
  let mut split_url = raw_url.split("=").collect::<Vec<_>>();
  split_url.pop();
  split_url.push(chapter_id.as_str());

  Ok(format!("{}{}",BASE_URL,split_url.join("=")))
}