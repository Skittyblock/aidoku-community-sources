use aidoku::{
	error::AidokuError,
	helpers::substring::Substring,
	prelude::format,
	std::{
		html::Node,
		net::{HttpMethod, Request},
		String,
	},
};

pub fn create_search_request_object(
	base_url: String,
	search_query: String,
) -> Result<Node, AidokuError> {
	let html = Request::new(base_url, HttpMethod::Get)
		.html()
		.expect("Failed to get html");
	let csrf_elem = html.select("meta[name=csrf-token]");
	let csrf = csrf_elem.attr("content").read();

	let request_info_elem = html.select("[wire:initial-data]");
	let request_info = request_info_elem.attr("wire:initial-data");
	let liveware = request_info.read();

	let fingerprint;
	match liveware.substring_after("{\"fingerprint\":") {
		Some(v) => match v.substring_before(",\"effects\"") {
			Some(w) => fingerprint = w,
			None => panic!(),
		},
		None => panic!(),
	}
	let name;
	match liveware.substring_after("\"name\":\"") {
		Some(v) => match v.substring_before("\",\"locale\"") {
			Some(w) => name = w,
			None => panic!(),
		},
		None => panic!(),
	}
	let servermemo;
	match liveware.substring_after(",\"serverMemo\":") {
		Some(v) => match v.substring_before_last("}") {
			Some(w) => servermemo = w,
			None => panic!(),
		},
		None => panic!(),
	}
	let mut updates = String::from(
		"[{\"type\":\"syncInput\",\"payload\":{\"id\":\"f6wl\",\"name\":\"query\",\"value\":\"",
	);
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
		.header(
			"User-Agent",
			"Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:108.0) Gecko/20100101 Firefox/108.0",
		)
		.header("Content-Type", "application/json")
		.header("X-Livewire", "true")
		.header("X-CSRF-TOKEN", &csrf);

	let json = req
		.json()
		.expect(
			"Error: Invalid Post-request response for search request. Couldn't convert to JSON.",
		)
		.as_object();

	let json_effects_html_string = json?
		.get("effects")
		.as_object()?
		.get("html")
		.as_string()?
		.read();
	// Convert String HTML to Node result.
	Node::new_fragment(json_effects_html_string)
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

	let fingerprint;
	match liveware.substring_after("{\"fingerprint\":") {
		Some(v) => match v.substring_before(",\"effects\"") {
			Some(w) => fingerprint = w,
			None => panic!(),
		},
		None => panic!(),
	}
	let name;
	match liveware.substring_after("\"name\":\"") {
		Some(v) => match v.substring_before("\",\"locale\"") {
			Some(w) => name = w,
			None => panic!(),
		},
		None => panic!(),
	}
	let servermemo;
	match liveware.substring_after(",\"serverMemo\":") {
		Some(v) => match v.substring_before_last("}") {
			Some(w) => servermemo = w,
			None => panic!(),
		},
		None => panic!(),
	}
	let mut updates = String::from(
		"[{\"type\":\"callMethod\",\"payload\":{\"id\":\"f8ay\",\"method\":\"gotoPage\",\"params\":[",
	);
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
		.header(
			"User-Agent",
			"Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:108.0) Gecko/20100101 Firefox/108.0",
		)
		.header("Content-Type", "application/json")
		.header("X-Livewire", "true")
		.header("X-CSRF-TOKEN", &csrf);

	let json = req
		.json()
		.expect(
			"Error: Invalid Post-request response for chapter request. Couldn't convert to JSON.",
		)
		.as_object();

	let json_effects_html_string = json?
		.get("effects")
		.as_object()?
		.get("html")
		.as_string()?
		.read();

	// Convert String HTML to Node result.
	Node::new_fragment(json_effects_html_string)
}
