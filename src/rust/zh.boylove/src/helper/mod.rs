pub mod setting;
pub mod url;

use aidoku::{
	error::Result,
	std::{ArrayRef, ObjectRef, Vec},
	Manga, MangaContentRating, MangaPageResult, MangaStatus,
};
use alloc::string::ToString as _;
use url::Url;

pub trait MangaListRes {
	fn get_manga_page_res(self) -> Result<MangaPageResult>;
}

impl MangaListRes for ObjectRef {
	fn get_manga_page_res(self) -> Result<MangaPageResult> {
		let manga = self.get("list").as_array()?.get_manga_list()?;

		let has_more = !self.get("lastPage").as_bool()?;

		Ok(MangaPageResult { manga, has_more })
	}
}

pub trait MangaList {
	fn get_manga_list(self) -> Result<Vec<Manga>>;
}

impl MangaList for ArrayRef {
	fn get_manga_list(self) -> Result<Vec<Manga>> {
		let manga = self
			.map(|val| {
				let item = val.as_object()?;
				let mut is_ad = item.get("lanmu_id").as_int().unwrap_or(-1) == 5;

				let id = item.get("id").as_int()?.to_string();
				match id.as_str() {
					"13286" | "13591" | "13677" | "14600" | "25532" => is_ad = true,
					_ => (),
				}

				if is_ad {
					return Ok(None);
				}

				let cover = item
					.get("image")
					.as_string()
					.map(|str_ref| {
						let path = &str_ref.read();

						Url::Abs { path }.into()
					})
					.unwrap_or_default();

				let title = item.get("title").as_string().unwrap_or_default().read();

				let author = item
					.get("auther")
					.as_string()
					.unwrap_or_default()
					.read()
					.replace(',', "、");

				let description = item.get("desc").as_string().unwrap_or_default().read();

				let url = Url::Manga { id: &id }.into();

				let mut nsfw = MangaContentRating::Nsfw;
				let categories = item
					.get("keyword")
					.as_string()
					.unwrap_or_default()
					.read()
					.split(',')
					.filter_map(|tag| {
						if tag == "清水" {
							nsfw = MangaContentRating::Safe;
						}

						(!tag.is_empty()).then(|| tag.into())
					})
					.collect();

				let status = match item.get("mhstatus").as_int()? {
					0 => MangaStatus::Ongoing,
					1 => MangaStatus::Completed,
					_ => MangaStatus::Unknown,
				};

				Ok(Some(Manga {
					id,
					cover,
					title,
					author,
					description,
					url,
					categories,
					status,
					nsfw,
					..Default::default()
				}))
			})
			.collect::<Result<Vec<_>>>()?
			.into_iter()
			.flatten()
			.collect();

		Ok(manga)
	}
}
