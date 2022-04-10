import { HostObject, Filter, Listing, Request, Chapter, Manga } from "aidoku-as";

import { TuMangaOnlineSource as Source } from "./TuMangaOnlineSource";

let source = new Source();

export function get_manga_list(filter_list_descriptor: i32, page: i32): i32 {
	let filters: Filter[] = [];
	let objects = new HostObject(filter_list_descriptor).toArray();
	for (let i = 0; i < objects.length; i++) filters.push(new Filter(objects[i]));
	let result = source.getMangaList(filters, page);
	return result.value;
}

export function get_manga_listing(listing: i32, page: i32): i32 {
	return source.getMangaListing(new Listing(listing), page).value;
}

export function get_manga_details(manga_descriptor: i32): i32 {
	let id = new HostObject(manga_descriptor).get("id").toString();
	return source.getMangaDetails(id).value;
}

export function get_chapter_list(manga_descriptor: i32): i32 {
	let id = new HostObject(manga_descriptor).get("id").toString();
	let array = HostObject.array();
	let result = source.getChapterList(id);
	for (let i = 0; i < result.length; i++) array.push(new HostObject(result[i].value));
	return array.rid;
}

export function get_page_list(chapter_descriptor: i32): i32 {
	let id = new HostObject(chapter_descriptor).get("id").toString();
	let array = HostObject.array();
	let result = source.getPageList(id);
	for (let i = 0; i < result.length; i++) array.push(new HostObject(result[i].value));
	return array.rid;
}

export function modify_image_request(req: i32): void {
	let request = new Request(req);
	source.modifyImageRequest(request);
}

export function handle_url(url: i32): i32 {
	let result = source.handleUrl(new HostObject(url).toString());
	if (result == null) return -1;
	if (result.chapter != null) {
		return (result.chapter as Chapter).value;
	} else if (result.manga != null) {
		return (result.manga as Manga).value;
	}
	return -1;
}
