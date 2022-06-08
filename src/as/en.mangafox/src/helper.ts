import {
    Manga,
    Html,
    MangaPageResult,
    MangaStatus,
    MangaContentRating,
    MangaViewer,
    Chapter,
    Page,
    Filter,
    FilterType,
    ValueRef,
} from 'aidoku-as/src';

export class Parser {
    parseHomePage(document: Html, listType: boolean): MangaPageResult {
        let result: Manga[] = [];
        // two different types of collections styles
        let ul = (listType) ? 'ul.manga-list-4-list > li' : 'ul.manga-list-1-list > li';
        let elements = document.select(ul).array();

        for (let i=0; i<elements.length; i++) {
            let elem    = elements[i];
            const id    = elem.select('a').attr('href').replace('/manga/', '').replace('/', '');
            const title = elem.select('a').attr('title')
            const img   = elem.select('img').attr('src');
            let manga   = new Manga(id, title);
            manga.cover_url = img;
            result.push(manga);
        }
        document.close();
        return new MangaPageResult(result, result.length != 0);
    }

    getMangaDetails(document: Html, mangaId: string): Manga{
        const title  = document.select('span.detail-info-right-title-font').text().trim()
        const author = document.select('p.detail-info-right-say a').text().trim();
        const desc   = document.select('p.fullcontent').text().trim();
        const image  = document.select('.detail-info-cover-img').attr('src');
        const stat   = document.select('.detail-info-right-title-tip').text().trim();

        let manga = new Manga(mangaId, title);
        manga.author      = author;
        manga.description = desc;
        manga.cover_url   = image;
        manga.status      = MangaStatus.Ongoing
        manga.url         = `https://fanfox.net/manga/${mangaId}`;
        if(stat == 'Ongoing')   manga.status = MangaStatus.Ongoing;
        if(stat == 'Completed') manga.status = MangaStatus.Completed;

        let nsfw = false;

        let tags: string[] = [];
        const genreArr = document.select('.detail-info-right-tag-list a').array();
        for (let i=0; i<genreArr.length; i++) {
            let genre = genreArr[i].text().trim();
            tags.push(genre);
            if (genre == 'Ecchi' || genre == 'Mature' || genre == 'Smut' || genre == 'Adult') nsfw = true;
        }
        manga.categories = tags;

        if (nsfw) manga.rating = MangaContentRating.NSFW;

        if (tags.includes("Webtoons")) manga.viewer = MangaViewer.Scroll;
        else manga.viewer = MangaViewer.RTL;

        document.close();
        return manga
    }

    getChapterList(document: Html, mangaId: string): Chapter[] {
        let chapters: Chapter[] = [];
        const elements  = document.select('div#chapterlist ul li').array();
        for (let i=0; i<elements.length; i++) {
            let element = elements[i];
            const url   = element.select('a').attr('href');
            const id    = url.replace('/manga/', '').replace('/1.html', '');
            const spl   = id.split('/');

            let title = '';
            const titleSplit = element.select('p.title3').text().trim().split('-');
            if (titleSplit.length >= 2) title = titleSplit.slice(1).join('-');

            let dateString = element.select('.title2').text().trim();
            let dateObject = ValueRef.string(dateString);
            let date = dateObject.toDate('MMM d,yyyy', 'en_US');
            dateObject.close();

            let chapter = new Chapter(id, title);
            chapter.url     = url;
            chapter.lang    = 'en';
            chapter.chapter = parseFloat(spl[spl.length-1].replace('c', '')) as f32;
            chapter.dateUpdated = date;

            if (spl.length > 2) {
                chapter.volume = parseFloat(spl[spl.length-2].replace('v', '')) as f32;
            }
            chapters.push(chapter);
        }
        document.close();
        return chapters;
    }

    getPageList(document: Html): Page[] {
        let pages: Page[] = [];
        const elements = document.select('#viewer > img').array();
        if (elements.length == 0) {
            const tmp = new Page(0);
            tmp.url = 'https://i.imgur.com/ovHuAps.png'
            pages.push(tmp);
        }
        for (let i=0; i<elements.length; i++) {
            let page   = new Page(i);
            let url    = elements[i].attr('data-original');
            if (url[0] == '/' &&
                url[1] == '/') url = 'https:' + url;
            page.url = url;
            pages.push(page);
        }
        document.close();
        return pages;
    }
}

export class FilterMap {
    private genreValues: Map<string, string>  = new Map<string, string>();
    private baseUrl: string;

    constructor(baseUrl: string) {
        this.baseUrl = baseUrl;
        this.genreValues.set('Action',         '1');
        this.genreValues.set('Adventure',      '2');
        this.genreValues.set('Comedy',         '3');
        this.genreValues.set('Drama',          '4');
        this.genreValues.set('Fantasy',        '5');
        this.genreValues.set('Martial Arts',   '6');
        this.genreValues.set('Shounen',        '7');
        this.genreValues.set('Horror',         '8');
        this.genreValues.set('Supernatural',   '9');
        this.genreValues.set('Harem',         '10');
        this.genreValues.set('Psychological', '11');
        this.genreValues.set('Romance',       '12');
        this.genreValues.set('School Life',   '13');
        this.genreValues.set('Shoujo',        '14');
        this.genreValues.set('Mystery',       '15');
        this.genreValues.set('Sci-fi',        '16');
        this.genreValues.set('Seinen',        '17');
        this.genreValues.set('Tragedy',       '18');
        this.genreValues.set('Ecchi',         '19');
        this.genreValues.set('Sports',        '20');
        this.genreValues.set('Slice of Life', '21');
        this.genreValues.set('Mature',        '22');
        this.genreValues.set('Shoujo Ai',     '23');
        this.genreValues.set('Webtoons',      '24');
        this.genreValues.set('Doujinshi',     '25');
        this.genreValues.set('One Shot',      '26');
        this.genreValues.set('Smut',          '27');
        this.genreValues.set('Yaoi',          '28');
        this.genreValues.set('Josei',         '29');
        this.genreValues.set('Historical',    '30');
        this.genreValues.set('Shounen Ai',    '31');
        this.genreValues.set('Gender Bender', '32');
        this.genreValues.set('Adult',         '33');
        this.genreValues.set('Yuri',          '34');
        this.genreValues.set('Mecha',         '35');
        this.genreValues.set('Lolicon',       '36');
        this.genreValues.set('Shotacon',      '37');
    }

    private getGenres(name: string): string {
        return this.genreValues.get(name);
    }

    getFilteredURL(filters: Filter[], page: number): string {
        let url = '';
        for (let i = 0; i < filters.length; i++) {
            const getIndex = (filters[i].value.toInteger() as i32)
            if (filters[i].type == FilterType.Title) {
                url += `&name=${filters[i].value.toString().trim().replace(' ', '+')}`;
            }
            else if (filters[i].name == 'Language' && getIndex != 0) {
                url += `&type=${getIndex}`;
            }
            else if (filters[i].name == 'Rating' && getIndex != 0) {
                url += `&rating_method=eq&rating=${getIndex}`;
            }
            else if (filters[i].name == 'Completed' && getIndex != 0) {
                url += `&st=${getIndex}`;
            }
            else if (filters[i].type == FilterType.Genre) {
                if (getIndex == 1) url += '&genres=' + this.getGenres(filters[i].name);
                if (getIndex == 0) url += '&nogenres=' + this.getGenres(filters[i].name);
            }
        }
        if (url.length > 0) {
            return `${this.baseUrl}/search?page=${page.toString().replace('.0', '')}${url}`;
        }
        return `${this.baseUrl}/directory/${page.toString().replace('.0', '')}.html`;
    }
}
