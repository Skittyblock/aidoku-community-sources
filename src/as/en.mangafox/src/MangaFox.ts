import {
    Source,
    Filter,
    Listing,
    MangaPageResult,
    Manga,
    Chapter,
    Page,
    Request,
    HttpMethod,
    // DeepLink,
} from 'aidoku-as/src';

import { Parser, FilterMap } from './helper';

const mangaFoxURL = 'https://fanfox.net';
const mangaFoxMOB = 'https://m.fanfox.net'

export class MangaFox extends Source {
    private baseUrl: string = mangaFoxURL;
    private mobUrl : string = mangaFoxMOB;
    private headers: Map<string, string>;
    private parser: Parser = new Parser();
    private filtermap: FilterMap = new FilterMap(this.baseUrl);

    constructor() {
        super();
        this.headers = new Map<string, string>();
        this.headers.set('Referer', this.baseUrl);
        this.headers.set('Cookie', 'isAdult=1');
    }

    getMangaList(filters: Filter[], page: number): MangaPageResult {
        const url   = this.filtermap.getFilteredURL(filters, page);
        let request     = Request.create(HttpMethod.GET);
        request.url     = url;
        request.headers = this.headers;
        return this.parser.parseHomePage(request.html(), (url.includes('search') == true));
    }

    getMangaListing(listing: Listing, page: number): MangaPageResult {
        let request     = Request.create(HttpMethod.GET);
        request.url = (listing.name == 'Latest') ? `${this.baseUrl}/releases/${page.toString().replace('.0', '')}.html` : `${this.baseUrl}/ranking`;
        request.headers = this.headers;
        return this.parser.parseHomePage(request.html(), (listing.name == 'Latest'));
    }

    getMangaDetails(mangaId: string): Manga {
        let request     = Request.create(HttpMethod.GET);
        request.url     = `${this.baseUrl}/manga/${mangaId}`;
        request.headers = this.headers;
        return this.parser.getMangaDetails(request.html(), mangaId);
    }

    getChapterList(mangaId: string): Chapter[] {
        let request     = Request.create(HttpMethod.GET);
        request.url     = `${this.baseUrl}/manga/${mangaId}`;
        request.headers = this.headers;
        return this.parser.getChapterList(request.html(), mangaId);
    }

    getPageList(chapterId: string): Page[] {
        let request     = Request.create(HttpMethod.GET);
        request.url     = `${this.mobUrl}/roll_manga/${chapterId}/1.html`;
        request.headers = this.headers;
        return this.parser.getPageList(request.html());
    }

    modifyImageRequest(request: Request): void {
        request.headers = this.headers;
    }

    // handleUrl(url: string): DeepLink | null {
    //     // [ 'https:', '', 'fanfox.net', 'manga', 'tales_of_demons_and_gods', '' ]
    //     const url_split = url.split('/');
    //     if (url_split[2] !== 'fanfox.net') return null;
    //     if (url_split[3] !== 'manga') return null;
    //     const mangaId: string = url_split[4];
    //     const manga: Manga = this.getMangaDetails(mangaId);

    //     const link: DeepLink = new DeepLink(manga, null);
    //     return link
    // }
}
