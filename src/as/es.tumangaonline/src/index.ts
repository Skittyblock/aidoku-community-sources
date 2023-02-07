import { ValueRef, ArrayRef, Filter, Listing, Request, Chapter, Manga, defaults } from "aidoku-as/src/index";

import { TuMangaOnlineSource as Source } from "./TuMangaOnlineSource";

let source = new Source();

@external("net", "set_rate_limit")
declare function set_rate_limit(rate_limit: i32): void;

@external("net", "set_rate_limit_period")
declare function set_rate_limit_period(period: i32): void;

export function initialize(): void {
	const rateLimit = defaults.get("rateLimit").toInteger();
	set_rate_limit(rateLimit as i32);
	set_rate_limit_period(60);
}

export function get_manga_list(filter_list_descriptor: i32, page: i32): i32 {
	let filters: Filter[] = [];
	let objects = new ValueRef(filter_list_descriptor).asArray().toArray();
	for (let i = 0; i < objects.length; i++) filters.push(new Filter(objects[i].asObject()));
	let result = source.getMangaList(filters, page);
	return result.value;
}

export function get_manga_listing(listing: i32, page: i32): i32 {
	return source.getMangaListing(new Listing(listing), page).value;
}

export function get_manga_details(manga_descriptor: i32): i32 {
	let id = new ValueRef(manga_descriptor).asObject().get("id").toString();
	return source.getMangaDetails(id).value;
}

export function get_chapter_list(manga_descriptor: i32): i32 {
	let id = new ValueRef(manga_descriptor).asObject().get("id").toString();
	let array = ArrayRef.new();
	let result = source.getChapterList(id);
	for (let i = 0; i < result.length; i++) array.push(new ValueRef(result[i].value));
	return array.value.rid;
}

export function get_page_list(chapter_descriptor: i32): i32 {
	let id = new ValueRef(chapter_descriptor).asObject().get("id").toString();
	let array = ArrayRef.new();
	let result = source.getPageList(id);
	for (let i = 0; i < result.length; i++) array.push(new ValueRef(result[i].value));
	return array.value.rid;
}

export function modify_image_request(req: i32): void {
	let request = new Request(req);
	source.modifyImageRequest(request);
}

export function handle_url(url: i32): i32 {
	let result = source.handleUrl(new ValueRef(url).toString());
	if (result == null) return -1;
	if (result.chapter != null) {
		return (result.chapter as Chapter).value;
	} else if (result.manga != null) {
		return (result.manga as Manga).value;
	}
	return -1;
}

export function handle_notification(notification: i32): void {
	let notif = new ValueRef(notification).toString();
	if (notif == "es.tumangaonline.rateLimit") {
		const rateLimit = defaults.get("rateLimit").toInteger();
		set_rate_limit(rateLimit as i32)
	}
}
