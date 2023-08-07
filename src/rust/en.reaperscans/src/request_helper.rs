use aidoku::{
	error::AidokuError,
	helpers::substring::Substring,
	prelude::format,
	std::html::Node,
	std::net::{HttpMethod, Request},
	std::{String, Vec},
};

use crate::helper::USER_AGENT;

pub fn create_search_request_object(
	base_url: String,
	search_query: String,
) -> Result<Node, AidokuError> {
	let html = Request::new(base_url, HttpMethod::Get).html()?;
	let csrf_elem = html.select("meta[name=csrf-token]");
	let csrf = csrf_elem.attr("content").read();

	let request_info_elem = html.select("[wire:initial-data]");
	let request_info = request_info_elem.attr("wire:initial-data");
	let liveware = request_info.read();

	// Livewire will only be empty if cloudflare is triggered
	// We return an error so the request can be retried,
	// and the user will be prompted to solve a captcha.
	if liveware.is_empty() {
		return Err(aidoku::error::AidokuError {
			reason: aidoku::error::AidokuErrorKind::NodeError(aidoku::error::NodeError::ParseError),
		});
	}

	let fingerprint = &liveware
		.substring_after("{\"fingerprint\":")
		.unwrap()
		.substring_before(",\"effects\"")
		.unwrap();

	let name = &fingerprint
		.substring_after("\"name\":\"")
		.unwrap()
		.substring_before("\",\"locale\"")
		.unwrap();

	let servermemo = &liveware
		.substring_after(",\"serverMemo\":")
		.unwrap()
		.substring_before_last("}")
		.unwrap();

	// TODO: Randomly generate 3-5 char id's from charset of a-z 0-9
	let ids = Vec::from([
		"xqbaj", "0qdw", "azp8", "qc8t", "ify7", "ny5wl", "jtml", "l9y3", "cpi4", "wqwx", "4p53",
		"yhmf", "re9d", "bskg", "w9zkk", "58ud", "qpwn", "hw68", "m1nd", "g5zn", "rjaj",
	]);

	let current_id = &ids[5];

	let mut updates = String::from("[{\"type\":\"syncInput\",\"payload\":{\"id\":\"");
	updates.push_str(current_id);
	updates.push_str("\",\"name\":\"query\",\"value\":\"");
	updates.push_str(&search_query);
	updates.push_str("\"}}]");

	// create POST request body.
	let mut body = String::from("{\"fingerprint\":");
	body.push_str(fingerprint);
	body.push_str(",\"serverMemo\":");
	body.push_str(servermemo);
	body.push_str(",\"updates\":");
	body.push_str(&updates);
	body.push('}');

	let post_url = format!("{}livewire/message/{}", html.base_uri(), name);
	let req = Request::new(post_url, HttpMethod::Post)
		.body(body.as_bytes())
		.header("User-Agent", USER_AGENT)
		.header("Content-Type", "application/json")
		.header("X-Livewire", "true")
		.header("X-CSRF-TOKEN", &csrf);

	// Return an error if the request could not be parsed as json because cloudflare
	// was triggered. This will cause the request to be retried, and the user will
	// be prompted to solve a captcha.
	if let Ok(json) = req.json() {
		let json_effects_html_string = json
			.as_object()?
			.get("effects")
			.as_object()?
			.get("html")
			.as_string()?
			.read();

		// Convert String HTML to Node result.
		Node::new_fragment(json_effects_html_string)
	} else {
		Err(aidoku::error::AidokuError {
			reason: aidoku::error::AidokuErrorKind::NodeError(aidoku::error::NodeError::ParseError),
		})
	}
}

pub fn create_chapter_request_object(
	html: Node,
	base_url: String,
	page: String,
) -> Result<Node, AidokuError> {
	let csrf_elem = html.select("meta[name=csrf-token]");
	let csrf = csrf_elem.attr("content").read();

	let request_info_elem = html.select("div[wire:initial-data*=Models\\\\Comic]");
	let request_info = request_info_elem.attr("wire:initial-data");
	let liveware = request_info.read();

	// Livewire will only be empty if cloudflare is triggered
	// We return an error so the request can be retried,
	// and the user will be prompted to solve a captcha.
	if liveware.is_empty() {
		return Err(aidoku::error::AidokuError {
			reason: aidoku::error::AidokuErrorKind::NodeError(aidoku::error::NodeError::ParseError),
		});
	}

	let fingerprint = &liveware
		.substring_after("{\"fingerprint\":")
		.unwrap()
		.substring_before(",\"effects\"")
		.unwrap();

	let name = &fingerprint
		.substring_after("\"name\":\"")
		.unwrap()
		.substring_before("\",\"locale\"")
		.unwrap();

	let servermemo = &liveware
		.substring_after(",\"serverMemo\":")
		.unwrap()
		.substring_before_last("}")
		.unwrap();

	// TODO: Randomly generate 3-5 char id's from charset of a-z 0-9
	let ids = Vec::from([
		"xqbaj", "0qdw", "azp8", "qc8t", "ify7", "ny5wl", "jtml", "l9y3", "cpi4", "wqwx", "4p53",
		"yhmf", "re9d", "bskg", "w9zkk", "58ud", "qpwn", "hw68", "m1nd", "g5zn", "rjaj",
	]);

	let current_id = &ids[page.parse::<usize>().unwrap() - 1];

	let mut updates = String::from("[{\"type\":\"callMethod\",\"payload\":{\"id\":\"");
	updates.push_str(current_id);
	updates.push_str("\",\"method\":\"gotoPage\",\"params\":[");
	updates.push_str(&page);
	updates.push_str(",\"page\"]}}]");

	// create POST request body.
	let mut body = String::from("{\"fingerprint\":");
	body.push_str(fingerprint);
	body.push_str(",\"serverMemo\":");
	body.push_str(servermemo);
	body.push_str(",\"updates\":");
	body.push_str(&updates);
	body.push('}');

	let post_url = format!("{}/livewire/message/{}", base_url, name);
	let req = Request::new(post_url, HttpMethod::Post)
		.body(body.as_bytes())
		.header("User-Agent", USER_AGENT)
		.header("Content-Type", "application/json")
		.header("X-Livewire", "true")
		.header("X-CSRF-TOKEN", &csrf);

	// Return an error if the request could not be parsed as json because cloudflare
	// was triggered. This will cause the request to be retried, and the user will
	// be prompted to solve a captcha.
	if let Ok(json) = req.json() {
		let json_effects_html_string = json
			.as_object()?
			.get("effects")
			.as_object()?
			.get("html")
			.as_string()?
			.read();

		// Convert String HTML to Node result.
		Node::new_fragment(json_effects_html_string)
	} else {
		Err(aidoku::error::AidokuError {
			reason: aidoku::error::AidokuErrorKind::NodeError(aidoku::error::NodeError::ParseError),
		})
	}
}
