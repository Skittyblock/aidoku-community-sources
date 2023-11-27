use aidoku::{
	helpers::{substring::*, uri::encode_uri_component},
	prelude::*,
	std::{String, Vec},
	Manga, MangaContentRating, MangaStatus, MangaViewer,
};

pub fn parse_chapter_and_title(text: String) -> Option<(f32, String)> {
	let title = text.substring_after(":")?;
	let chapter = text
		.split(':')
		.rev()
		.find_map(|p| p.substring_after("Comic ")?.parse::<f32>().ok())?;
	Some((chapter, String::from(title)))
}

pub fn word_wrap(text: String) -> String {
	let mut res = String::new();
	text.replace("\r\n", " ")
		.replace(['\r', '\n'], " ")
		.split(' ')
		.enumerate()
		.for_each(|(i, w)| {
			if i > 0 && i % 7 == 0 {
				res.push('\n');
			}
			res.push_str(w);
			res.push(' ');
		});
	res
}

pub fn newsblip_image_url(text: String) -> String {
	let text = encode_uri_component(word_wrap(text));
	format!("https://fakeimg.pl/1500x2126/ffffff/000000/?font=noto&font_size=42&text={text}")
}

pub fn comic_info() -> Manga {
	Manga {
		id: String::from("en.questionablecontent"),
		cover: String::from("https://upload.wikimedia.org/wikipedia/en/2/26/Questionable_content.png"),
		title: String::from("Questionable Content"),
		author: String::from("Jeph Jacques"),
		artist: String::from("Jeph Jacques"),
		description: String::from("Questionable Content is an internet comic strip about friendship, romance, and robots.\n\nThe world of QC is set in the present day (whenever the present day actually is) and is pretty much the same as our own except there are robots all over the place and giant space stations and the United States wasn't ravaged by a pandemic in 2020 and...okay so there are some differences. But it's not too far off!!! Anyway it's set in Northampton, Massachusetts and follows best buddies Marten and Faye as they navigate life, make friends and forge relationships, and there is definitely some robot smoochin' later on. If the giant archive of comics intimidates you, don't worry- you can pretty much jump in anywhere and have a general idea of what's going on in a dozen strips or so. If you're looking for what I, THE AUTHOR, would recommend as a good place to start, I'd say 3500 is a pretty good jumping in point for the current state of the comic.
Fun facts: QC started on August 1, 2003. There are a whole bunch of horrible alternate URLS you can use to navigate to the comic."),
		url: String::from("https://questionablecontent.net"),
		categories: Vec::new(),
		status: MangaStatus::Ongoing,
		nsfw: MangaContentRating::Safe,
		viewer: MangaViewer::Ltr,
	}
}
