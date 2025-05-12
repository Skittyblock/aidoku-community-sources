pub mod setting;
pub mod url;

use aidoku::{
	error::{AidokuError, AidokuErrorKind, Result},
	prelude::println,
	std::{ArrayRef, ObjectRef, Vec},
	Manga, MangaContentRating, MangaPageResult, MangaStatus,
};
use alloc::string::ToString as _;
use chinese_number::{ChineseCountMethod, ChineseToNumber as _};
use core::str::FromStr;
use regex::Regex as OriginalRegex;
use url::Url;

pub struct Regex;

impl Regex {
	#[expect(clippy::new_ret_no_self)]
	pub fn new<S: AsRef<str>>(re: S) -> Result<OriginalRegex> {
		OriginalRegex::new(re.as_ref()).map_err(|e| {
			println!("{e}");

			AidokuError {
				reason: AidokuErrorKind::Unimplemented,
			}
		})
	}
}

pub struct Part {
	pub volume: f32,
	pub chapter: f32,
}

impl Default for Part {
	fn default() -> Self {
		let volume = -1.0;

		let chapter = -1.0;

		Self { volume, chapter }
	}
}

impl FromStr for Part {
	type Err = AidokuError;

	fn from_str(title: &str) -> Result<Self> {
		if let Some(caps) = Regex::new("^全[一1](?<type>[卷話话回])$")?.captures(title) {
			if &caps["type"] == "卷" {
				let volume = 1.0;

				let chapter = -1.0;

				return Ok(Self { volume, chapter });
			}

			let volume = -1.0;

			let chapter = 1.0;

			return Ok(Self { volume, chapter });
		}

		let pat = r"^(第?(?<volume>[\d零一二三四五六七八九十百千]+(\.\d+)?)[卷部季] ?)?(第?(?<chapter>[\d零一二三四五六七八九十百千]+(\.\d+)?)(-(\d+(\.\d+)?))?[话話回]?([(（].*[)）]|完结|END)?)?([ +]|$)";
		let Some(caps) = Regex::new(pat)?.captures(title) else {
			return Ok(Self::default());
		};
		let get_group = |name| {
			caps.name(name)
				.and_then(|m| {
					let str = m.as_str();

					str.parse()
						.ok()
						.or_else(|| str.to_number(ChineseCountMethod::TenThousand).ok())
				})
				.unwrap_or(-1.0)
		};
		let volume = get_group("volume");

		let chapter = get_group("chapter");

		Ok(Self { volume, chapter })
	}
}

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
						let path = str_ref.read();

						if path.starts_with('/') {
							return Url::Abs { path: &path }.into();
						}

						path
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
