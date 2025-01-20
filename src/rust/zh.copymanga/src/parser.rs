use crate::{
	helper::{to_aidoku_error, Regex},
	url::Url,
};
use aidoku::{
	error::{AidokuError, Result},
	prelude::format,
	std::{html::Node, json, ArrayRef, ObjectRef, String, ValueRef, Vec},
	Manga, MangaPageResult, MangaStatus,
};
use alloc::string::ToString;
use chinese_number::{ChineseCountMethod, ChineseToNumber as _};
use core::str::FromStr;
use uuid::Uuid;

pub trait MangaListResponse {
	fn get_page_result(self) -> Result<MangaPageResult>;
}

impl MangaListResponse for Node {
	fn get_page_result(self) -> Result<MangaPageResult> {
		let manga = self
			.get_attr("div.exemptComic-box", "list")
			.replace(r"\xa0", " ")
			.split('"')
			.enumerate()
			.map(|(index, str)| {
				if index % 2 == 0 {
					str.replace('\'', "\"")
				} else if !str.contains('\'') {
					format!(r"\{str}\")
				} else {
					str.to_string()
				}
			})
			.collect::<Vec<_>>()
			.join("\"")
			.replace(r#""\"#, r#"\""#)
			.json()?
			.as_array()?
			.get_manga_list()?;

		let has_more = !self.select("li.page-all-item").last().has_class("active");

		Ok(MangaPageResult { manga, has_more })
	}
}

impl MangaListResponse for ValueRef {
	fn get_page_result(self) -> Result<MangaPageResult> {
		let results_obj = self.as_object()?.get("results").as_object()?;

		let manga = results_obj.get("list").as_array()?.get_manga_list()?;

		let total = results_obj.get("total").as_int()?;
		let limit = results_obj.get("limit").as_int()?;
		let offset = results_obj.get("offset").as_int()?;
		let has_more = (offset + limit) < total;

		Ok(MangaPageResult { manga, has_more })
	}
}

trait MangaArr {
	fn get_manga_list(self) -> Result<Vec<Manga>>;
}

impl MangaArr for ArrayRef {
	fn get_manga_list(self) -> Result<Vec<Manga>> {
		let mut manga = Vec::<Manga>::new();
		for manga_value in self {
			let manga_obj = manga_value.as_object()?;

			let manga_id = manga_obj.get_as_string("path_word")?;

			let cover = manga_obj
				.get_as_string("cover")?
				.replace(".328x422.jpg", "");

			let title = manga_obj.get_as_string("name")?;

			let artist = manga_obj
				.get("author")
				.as_array()?
				.filter_map(|value| value.as_object().ok())
				.filter_map(|obj| obj.get("name").as_string().ok())
				.map(|str_ref| str_ref.read())
				.collect::<Vec<_>>()
				.join("、");

			let manga_url = Url::Manga { id: &manga_id }.to_string();

			let status_code = manga_obj.get("status").as_int().unwrap_or(-1);
			let status = match status_code {
				0 => MangaStatus::Ongoing,
				1 | 2 => MangaStatus::Completed,
				_ => MangaStatus::Unknown,
			};

			manga.push(Manga {
				id: manga_id,
				cover,
				title,
				author: artist.clone(),
				artist,
				url: manga_url,
				status,
				..Default::default()
			});
		}

		Ok(manga)
	}
}

pub trait Element {
	fn get_attr(&self, selector: &str, attr: &str) -> String;
	fn get_text(&self, selector: &str) -> String;
}

impl Element for Node {
	fn get_attr(&self, selector: &str, attr: &str) -> String {
		self.select(selector).attr(attr).read()
	}

	fn get_text(&self, selector: &str) -> String {
		self.select(selector).text().read()
	}
}

pub trait NodeArrValue {
	fn ok_text(self) -> Option<String>;
}

impl NodeArrValue for ValueRef {
	fn ok_text(self) -> Option<String> {
		self.as_node().map(|node| node.text().read()).ok()
	}
}

pub trait JsonString {
	fn json(self) -> Result<ValueRef>;
}

impl JsonString for String {
	fn json(self) -> Result<ValueRef> {
		json::parse(self)
	}
}

pub trait JsonObj {
	fn get_as_string(&self, key: &str) -> Result<String>;
}

impl JsonObj for ObjectRef {
	fn get_as_string(&self, key: &str) -> Result<String> {
		Ok(self.get(key).as_string()?.read())
	}
}

pub trait UuidString {
	fn get_timestamp(&self) -> Result<f64>;
}

impl UuidString for String {
	fn get_timestamp(&self) -> Result<f64> {
		let (integer_part, fractional_part) = Uuid::from_str(self)
			.map_err(to_aidoku_error)?
			.get_timestamp()
			.ok_or_else(|| to_aidoku_error("Failed to parse UUID to timestamp."))?
			.to_unix();

		Ok((integer_part as f64) + (fractional_part as f64 * 10e-10))
	}
}

pub struct Part {
	pub volume: f32,
	pub chapter: f32,
	pub title: String,
}

impl FromStr for Part {
	type Err = AidokuError;

	fn from_str(title: &str) -> Result<Self> {
		match title {
			"全一卷" => {
				return Ok(Self {
					volume: 1.0,
					chapter: -1.0,
					title: title.into(),
				})
			}

			"全一話" | "全一话" => {
				return Ok(Self {
					volume: -1.0,
					chapter: 1.0,
					title: title.into(),
				})
			}

			_ => (),
		}

		let re = Regex::new(
			r"^(单行本：)?(第?(?<volume>[\d零一二三四五六七八九十百千]+(\.\d)?)[卷部季]完?)?((第|连载|CH)?(?<chapter>[\d零一二三四五六七八九十百千]+([\.-]\d+)?)[話话回]?(-?[(（]?(?<part>([前中后上下]|\d+))[)）]?篇?)?(试看)?)?(\s.*|$)",
		)?;
		let Some(caps) = re.captures(title) else {
			return Ok(Self {
				volume: -1.0,
				chapter: -1.0,
				title: title.into(),
			});
		};

		let get_option_number = |name: &str| {
			caps.name(name).map(|m| m.as_str()).and_then(|str| {
				str.parse().map_or_else(
					|_| str.to_number(ChineseCountMethod::TenThousand).ok(),
					Some,
				)
			})
		};

		let volume = get_option_number("volume").unwrap_or(-1.0);

		let part = caps
			.name("part")
			.map(|m| match m.as_str() {
				"前" | "上" => "0",
				"中" => "25",
				"后" | "下" => "5",
				digit => digit,
			})
			.unwrap_or("0");
		let chapter = get_option_number("chapter")
			.and_then(|num| {
				format!("{}{}{}", num, if num % 1.0 == 0.0 { "." } else { "" }, part)
					.parse()
					.ok()
			})
			.unwrap_or(-1.0);

		Ok(Self {
			volume,
			chapter,
			title: title.into(),
		})
	}
}
