import {
  Chapter,
  console,
  DeepLink,
  Filter,
  HttpMethod,
  Listing,
  Manga,
  MangaPageResult,
  Page,
  Request,
  Source,
} from "aidoku-as/src/index";
import { Parser, Search } from "./parser";
import { Constants } from "./constants";

export class HentaiVN extends Source {
  private baseUrl: string = Constants.domain;
  private headers: Map<string, string>;
  private parser: Parser = new Parser();
  private search: Search = new Search(this.baseUrl);

  constructor() {
    super();
    this.headers = new Map<string, string>();
    this.headers.set("Referer", this.baseUrl);
    this.headers.set("User-Agent", Constants.userAgent);
  }

  private logRequest(prefix: string, method: string, request: Request): void {
    console.log(`${prefix} ${method} ${request.url}`);
  }

  modifyImageRequest(request: Request): void {
    request.headers = this.headers;
  }

  getMangaList(filters: Filter[], page: i32): MangaPageResult {
    const url = this.search.getFilteredURL(filters, page);
    const resp = Request.create(HttpMethod.GET);
    resp.url = url ? url : this.baseUrl;
    resp.headers = this.headers;
    this.logRequest("[HentaiVN.getMangaList]", "GET", resp);
    return this.parser.parseSearchPage(resp.html(), page);
  }

  getMangaListing(listing: Listing, page: i32): MangaPageResult {
    const resp = Request.create(HttpMethod.GET);
    resp.headers = this.headers;
    if (listing.name == "New") {
      resp.url = `${this.baseUrl}/chap-moi.html?page=${
        page.toString().replaceAll(".0", "")
      }`;
    } else {
      resp.url = `${this.baseUrl}/da-hoan-thanh.html?page=${
        page.toString().replaceAll(".0", "")
      }`;
    }
    this.logRequest("[HentaiVN.getMangaListing]", "GET", resp);
    return this.parser.parseNewOrCompletePage(resp.html());
  }

  getMangaDetails(mangaId: string): Manga {
    const resp = Request.create(HttpMethod.GET);
    resp.url = `${this.baseUrl}/${mangaId}`;
    resp.headers = this.headers;
    this.logRequest("[HentaiVN.getMangaDetails]", "GET", resp);
    return this.parser.getMangaDetails(resp.html(), mangaId);
  }

  getChapterList(mangaId: string): Chapter[] {
    const resp = Request.create(HttpMethod.GET);
    const tempManga = mangaId.replaceAll(".html", "").split("-").filter((
      value: string,
    ) => !["doc", "truyen"].includes(value));
    resp.url =
      `${this.baseUrl}/list-showchapter.php?idchapshow=${tempManga.shift()}&idlinkanime=${
        tempManga.join("-")
      }`;
    resp.headers = this.headers;
    this.logRequest("[HentaiVN.getChapterList]", "GET", resp);
    return this.parser.getChapterList(resp.html());
  }

  getPageList(chapterId: string): Page[] {
    const resp = Request.create(HttpMethod.GET);
    resp.url = chapterId;
    resp.headers = this.headers;
    this.logRequest("[HentaiVN.getPageList]", "GET", resp);
    return this.parser.getPageList(resp.html());
  }

  private getMangaDetailsFromChapterPage(chapterId: string): Manga {
    const resp = Request.create(HttpMethod.GET);
    resp.url = chapterId;
    resp.headers = this.headers;
    this.logRequest("[HentaiVN.getMangaDetailsFromChapterPage]", "GET", resp);
    const document = resp.html();
    const href = document.select(
      "div.bar-title-episode:contains(Xem thông tin) > a",
    ).attr("href");
    console.log(
      `[HentaiVN.getMangaDetailsFromChapterPage] Extracted URL ${href}`,
    );
    document.close();
    const mangaId = href.split("/").pop();
    return this.getMangaDetails(mangaId);
  }

  handleUrl(url: string): DeepLink | null {
    // https://hentaivn.moe/24706-doc-truyen-dieu-duong-tuoi-dep-nguyen-tac.html
    // ['https:', '', 'hentaivn.moe', '24706-doc-truyen-dieu-duong-tuoi-dep-nguyen-tac.html']
    //
    console.log(`[HentaiVN.handleUrl] Received URL ${url}`);
    const mangaOrChapterId = url.split("/").pop();
    if (mangaOrChapterId.includes("doc-truyen")) {
      const manga = this.getMangaDetails(mangaOrChapterId);
      return new DeepLink(manga, null);
    } else {
      const manga = this.getMangaDetailsFromChapterPage(url);
      const chapters = this.getChapterList(manga.id);
      for (let i = 0; i < chapters.length; i++) {
        if (chapters[i].id == url) {
          return new DeepLink(manga, chapters[i]);
        }
      }
    }
    return null;
  }
}
