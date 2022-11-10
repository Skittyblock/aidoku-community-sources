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

const manhuaguiURL = "https://www.manhuagui.com";
const mobileURL = "https://m.manhuagui.com";

export class Manhuagui extends Source {
  private baseUrl: string = manhuaguiURL;
  private mobileURL: string = mobileURL;
  private headers: Map<string, string>;
  private parser: Parser = new Parser();
  private filterMap: FilterMap = new FilterMap(this.baseUrl);

  constructor() {
    super();
    this.headers = new Map<string, string>();
    this.headers.set("Referer", "https://www.manhuagui.com/");
    this.headers.set(
      "user-agent",
      "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/106.0.0.0 Safari/537.36"
    );
  }

  modifyImageRequest(request: Request): void {
    request.headers = this.headers;
  }

  getMangaList(filters: Filter[], page: i32): MangaPageResult {
    // TODO: implement search
    const url = this.filterMap.getFilteredURL(filters, page);
    let request = Request.create(HttpMethod.GET);
    request.url = url;
    request.headers = this.headers;
    return this.parser.parseHomePage(request.html());
  }

  getMangaListing(listing: Listing, page: i32): MangaPageResult {
    const url = this.filterMap.getFilteredURL([], page);
    let request = Request.create(HttpMethod.GET);
    request.url = url;
    request.headers = this.headers;
    return this.parser.parseHomePage(request.html());
  }

  getMangaDetails(mangaId: string): Manga {
    let request = Request.create(HttpMethod.GET);
    request.url = `${this.baseUrl}/comic/${mangaId}`;
    request.headers = this.headers;
    return this.parser.getMangaDetails(request.html(), mangaId);
  }

  getChapterList(mangaId: string): Chapter[] {
    let request = Request.create(HttpMethod.GET);
    request.url = `${this.baseUrl}/comic/${mangaId}`;
    request.headers = this.headers;
    return this.parser.getChapterList(request.html(), mangaId);
  }

  getPageList(chapterId: string): Page[] {
    let mobileUrl = `${this.mobileURL}/comic/${chapterId}`;
    let webUrl = `${this.baseUrl}/comic/${chapterId}`;
    return this.parser.getPageList(webUrl, mobileUrl);
  }

  private getMangaDetailsFromChapterPage(chapterId: string): Manga {
    // TODO
    return new Manga("1", "1");
  }

  // handleUrl(url: string): DeepLink | null {
  // 	// TODO
  // }
}
