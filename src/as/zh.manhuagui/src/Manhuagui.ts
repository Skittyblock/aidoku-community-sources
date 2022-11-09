import {
	Chapter,
	DeepLink,
	Filter,
	HttpMethod,
	Listing,
	Manga,
	MangaPageResult,
	Page,
	Request,
	Source,
} from "aidoku-as/src";
import { id } from "aidoku-as/src/modules/html";
import { FilterMap, Parser } from "./helper";

const manhuaguiURL = 'https://www.manhuagui.com';

export class Manhuagui extends Source {
	private baseUrl: string = manhuaguiURL;
	private headers: Map<string, string>;
	private parser: Parser = new Parser();
	private filterMap: FilterMap = new FilterMap(this.baseUrl)

	constructor() {
		super();
		this.headers = new Map<string, string>();
		this.headers.set("Referer", "https://www.manhuagui.com/")
	}

	modifyImageRequest(request: Request): void {
		request.headers = this.headers
	}

	getMangaList(filters: Filter[], page: i32): MangaPageResult {
		const url = this.filterMap.getFilteredURL(filters, page);
		let request     = Request.create(HttpMethod.GET);
        request.url     = url;
        request.headers = this.headers;
		return this.parser.parseHomePage(request.html())
	}

	getMangaListing(listing: Listing, page: i32): MangaPageResult {
		const url = this.filterMap.getFilteredURL([], page);
		let request     = Request.create(HttpMethod.GET);
        request.url     = url;
        request.headers = this.headers;
		return this.parser.parseHomePage(request.html())
	}

	getMangaDetails(mangaId: string): Manga {
		let request     = Request.create(HttpMethod.GET);
        request.url     = `${this.baseUrl}/comic/${mangaId}`;
        request.headers = this.headers;
        return this.parser.getMangaDetails(request.html(), mangaId);
	}

	getChapterList(mangaId: string): Chapter[] {
		let request     = Request.create(HttpMethod.GET);
        request.url     = `${this.baseUrl}/comic/${mangaId}`;
        request.headers = this.headers;
        return this.parser.getChapterList(request.html(), mangaId);
	}

	getPageList(chapterId: string): Page[] {
		// TODO
		let pages: Page[] = []
		pages.push(new Page(1))
		return pages
	}

	private getMangaDetailsFromChapterPage(chapterId: string): Manga {
		// TODO
		return new Manga("1","1")
	}

	// handleUrl(url: string): DeepLink | null {
	// 	// TODO
	// }
}
