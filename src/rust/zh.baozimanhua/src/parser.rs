use aidoku::{
	error::Result,
	helpers::substring::Substring,
	std::{html::Node, Vec},
	Manga,
};
use alloc::string::ToString;

pub trait DivComicsCard {
	fn get_manga_list(self) -> Result<Vec<Manga>>;
}

impl DivComicsCard for Node {
	fn get_manga_list(self) -> Result<Vec<Manga>> {
		self.array()
			.map(|value| {
				let div = value.as_node()?;

				let url = div.select("a.comics-card__poster").attr("abs:href").read();

				let id = url
					.substring_after_last('/')
					.expect("Unable to get the substring after the last '/'")
					.to_string();

				let cover = {
					let resized_cover = div.select("amp-img[noloading]").attr("src").read();
					resized_cover
						.clone()
						.substring_before_last('?')
						.map_or(resized_cover, ToString::to_string)
				};

				let title = div.select("h3").text().read();

				let artist = {
					let mut artists = div
						.select("small")
						.text()
						.read()
						.split(',')
						.map(ToString::to_string)
						.collect::<Vec<_>>();
					artists.dedup();

					artists.join("、")
				};

				let categories = div
					.select("span")
					.array()
					.map(|value| {
						let genre = value.as_node()?.text().read();
						Ok(genre)
					})
					.collect::<Result<_>>()?;

				Ok(Manga {
					id,
					cover,
					title,
					author: artist.clone(),
					artist,
					url,
					categories,
					..Default::default()
				})
			})
			.collect()
	}
}
