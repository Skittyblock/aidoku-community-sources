import {
	Source,
	FilterType,
	Filter,
	Listing,
	MangaPageResult,
	Manga,
	MangaStatus,
	MangaViewer,
	Chapter,
	Page,
	Request,
	HttpMethod,
	Html,
	ValueRef
} from "aidoku-as/src";

let genreValues = new Map<string, string>();
genreValues.set("Acción", "1");
genreValues.set("Aventura", "2");
genreValues.set("Comedia", "3");
genreValues.set("Drama", "4");
genreValues.set("Recuentos de la vida", "5");
genreValues.set("Ecchi", "6");
genreValues.set("Fantasia", "7");
genreValues.set("Magia", "8");
genreValues.set("Sobrenatural", "9");
genreValues.set("Horror", "10");
genreValues.set("Misterio", "11");
genreValues.set("Psicológico", "12");
genreValues.set("Romance", "13");
genreValues.set("Ciencia Ficción", "14");
genreValues.set("Thriller", "15");
genreValues.set("Deporte", "16");
genreValues.set("Girls Love", "17");
genreValues.set("Boys Love", "18");
genreValues.set("Harem", "19");
genreValues.set("Mecha", "20");
genreValues.set("Supervivencia", "21");
genreValues.set("Reencarnación", "22");
genreValues.set("Gore", "23");
genreValues.set("Apocalíptico", "24");
genreValues.set("Tragedia", "25");
genreValues.set("Vida Escolar", "26");
genreValues.set("Historia", "27");
genreValues.set("Militar", "28");
genreValues.set("Policiaco", "29");
genreValues.set("Crimen", "30");
genreValues.set("Superpoderes", "31");
genreValues.set("Vampiros", "32");
genreValues.set("Artes Marciales", "33");
genreValues.set("Samurái", "34");
genreValues.set("Género Bender", "35");
genreValues.set("Realidad Virtual", "36");
genreValues.set("Ciberpunk", "37");
genreValues.set("Musica", "38");
genreValues.set("Parodia", "39");
genreValues.set("Animación", "40");
genreValues.set("Demonios", "41");
genreValues.set("Familia", "42");
genreValues.set("Extranjero", "43");
genreValues.set("Niños", "44");
genreValues.set("Realidad", "45");
genreValues.set("Telenovela", "46");
genreValues.set("Guerra", "47");
genreValues.set("Oeste", "48");

let sortOptions = [ "likes_count", "alphabetically", "score", "creation", "release_date", "num_chapters" ];
let filterByOptions = ["title", "author", "company"];
let demographicOptions = ["", "seinen", "shoujo", "shounen", "josei", "kodomo"];
let statusOptions = ["", "publishing", "ended", "cancelled", "on_hold"];
let typeOptions = ["", "manga", "manhua", "manhwa", "novel", "one_shot", "doujinshi", "oel"];

let CACHED_MANGA_ID = "";
let CACHED_MANGA: ArrayBuffer = new ArrayBuffer(1);

function cache_manga_page(url: string, headers: Map<string, string>): void {
	if (CACHED_MANGA.byteLength > 1 && CACHED_MANGA_ID == url) {
		return
	}

	let request = Request.create(HttpMethod.GET);
	CACHED_MANGA_ID = request.url = url;
	request.headers = headers;

	CACHED_MANGA = request.data();
}

export class TuMangaOnlineSource extends Source {
	private headers: Map<string, string>;

	constructor() {
		super();
		this.headers = new Map<string, string>();
		this.headers.set("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/80.0.3987.132 Safari/537.36");
		this.headers.set("Referer", "https://lectortmo.com/");
	}

	private parseMangaList(url: string): MangaPageResult {
		let request = Request.create(HttpMethod.GET);
		request.url = url;
		request.headers = this.headers;
		let document = request.html();

		let elements = document.select("div.element").array();

		let result: Manga[] = [];

		for (let i = 0; i < elements.length; i++) {
			let element = elements[i].select("a").first();
			let id = element.attr("href").replace(" ", "");
			let title = element.select("h4.text-truncate").text();
			let manga = new Manga(id, title);

			let style = element.select("style").html();
			manga.cover_url = style.substring(
				style.indexOf("('") + 2, 
				style.lastIndexOf("')")
			);

			result.push(manga);
		}

		document.close();

		return new MangaPageResult(result, result.length != 0);
	}

	getMangaList(filters: Filter[], page: number): MangaPageResult {
		let url = "https://lectortmo.com/library?_pg=1&page=" + page.toString().replace(".0", "");

		for (let i = 0; i < filters.length; i++) {
			if (filters[i].type == FilterType.Title) {
				url += "&filter_by=title&title=" + filters[i].value.toString();
			} else if (filters[i].type == FilterType.Sort) {
				let option = filters[i].value.asObject();
				let ascending = option.get("ascending").toBool();
				let index = option.get("index").toInteger() as i32;
				if (index == -1) continue;
				url += "&order_item=" + sortOptions[index];
				url += "&order_type=" + (ascending ? "asc" : "desc");
			} else if (filters[i].name == "Filtrar por") {
				let value = filters[i].value.toInteger() as i32;
				if (value == -1) continue;
				url += "&filter_by=" + filterByOptions[value];
			} else if (filters[i].name == "Demografía") {
				let value = filters[i].value.toInteger() as i32;
				if (value <= 0) continue;
				url += "&demography=" + demographicOptions[value];
			} else if (filters[i].name == "Estado de traducción") {
				let value = filters[i].value.toInteger() as i32;
				if (value <= 0) continue;
				url += "&translation_status=" + statusOptions[value];
			} else if (filters[i].name == "Estado de serie") {
				let value = filters[i].value.toInteger() as i32;
				if (value <= 0) continue;
				url += "&status=" + statusOptions[value];
			} else if (filters[i].name == "Tipo") {
				let value = filters[i].value.toInteger() as i32;
				if (value <= 0) continue;
				url += "&type=" + typeOptions[value];
			} else if (filters[i].type == FilterType.Check) {
				let value = filters[i].value.toInteger();
				if (value == -1) continue;
				if (filters[i].name == "Webcomic") {
					url += "&webcomic=" + (value == 0 ? "false" : "true");
				} else if (filters[i].name == "Yonkoma") {
					url += "&yonkoma=" + (value == 0 ? "false" : "true");
				} else if (filters[i].name == "Amateur") {
					url += "&amateur=" + (value == 0 ? "false" : "true");
				} else if (filters[i].name == "Erótico") {
					url += "&erotic=" + (value == 0 ? "false" : "true");
				}
			} else if (filters[i].type == FilterType.Genre) {
				let value = filters[i].value.toInteger();
				if (value == 1) url += "&genders[]=" + genreValues.get(filters[i].name);
				else if (value == 0) url += "&exclude_genders[]=" + genreValues.get(filters[i].name);
			}
		}

		return this.parseMangaList(url);
	}

	getMangaListing(listing: Listing, page: number): MangaPageResult {
		if (listing.name == "Latest") {
			return this.parseMangaList("https://lectortmo.com/library?order_item=creation&order_dir=desc&filter_by=title&_pg=1&page=" + page.toString().replace(".0", ""));
		} else if (listing.name == "Popular") {
			return this.parseMangaList("https://lectortmo.com/library?order_item=likes_count&order_dir=desc&filter_by=title&_pg=1&page=" + page.toString().replace(".0", ""));
		}

		return this.getMangaList([], page);
	}

	getMangaDetails(mangaId: string): Manga {
		cache_manga_page(mangaId, this.headers);
		let document = Html.parse(CACHED_MANGA);
		
		let title = document.select("h2.element-subtitle").first().text();
		let titleElements = document.select("h5.card-title");
		let status = document.select("span.book-status").text();
		let type = document.select("h1.book-type").text();

		let mangaDetails = new Manga(mangaId, title);
		mangaDetails.url = mangaId;
		mangaDetails.cover_url = document.select(".book-thumbnail").attr("src");
		mangaDetails.author = titleElements.first().attr("title").replace(", ", "");
		mangaDetails.artist = titleElements.last().attr("title").replace(", ", "");
		mangaDetails.description = document.select("p.element-description").text();
		if (type.includes("MANHWA") || type.includes("MANHUA")) mangaDetails.viewer = MangaViewer.Scroll;
		else mangaDetails.viewer = MangaViewer.RTL;

		let tags: string[] = [];
		let genres = document.select("a.py-2").array();
		for (let i = 0; i < genres.length; i++) {
			tags.push(genres[i].text());
		}
		mangaDetails.categories = tags;

		if (status.includes("Publicándose")) mangaDetails.status = MangaStatus.Ongoing;
		else if (status.includes("Finalizado")) mangaDetails.status = MangaStatus.Completed;
		else mangaDetails.status = MangaStatus.Unknown;

		document.close();

		return mangaDetails;
	}

	private parseChapter(element: Html): Chapter {
		let url = element.select("div.row > .text-right > a").attr("href");

		let dateString = element.select("span.badge.badge-primary.p-2").first().text();
		let dateObject = ValueRef.string(dateString);
		let date = dateObject.toDate("yyyy-MM-dd");
		dateObject.close();

		let chapter = new Chapter(url, "");
		chapter.url = url;
		chapter.scanlator = element.select("div.col-md-6.text-truncate").text();
		chapter.dateUpdated = date;
		return chapter;
	}

	getChapterList(mangaId: string): Chapter[] {
		cache_manga_page(mangaId, this.headers);
		let document = Html.parse(CACHED_MANGA);

		let chapterElements = document.select("div.chapters > ul.list-group li.p-0.list-group-item").array();

		let chapters: Chapter[] = [];

		// One shot
		if (chapterElements.length == 0) {
			let elements = document.select("div.chapter-list-element > ul.list-group li.list-group-item").array();
			for (let i = 0; i < elements.length; i++) {
				let chapter = this.parseChapter(elements[i]);
				chapter.title = "One Shot";
				chapters.push(chapter);
			}
		} else {
			for (let i = 0; i < chapterElements.length; i++) {
				let numText = chapterElements[i].select("a.btn-collapse").text();
				let title = chapterElements[i].select("div.col-10.text-truncate").text();
				let chapterNum = parseInt(numText.substring(
					numText.indexOf("Capítulo ") + 9,
					numText.indexOf(":") < 0 ? numText.length - 1 : numText.indexOf(":")
				)) as f32;

				let scanlations = chapterElements[i].select("ul.chapter-list > li").array();

				for (let j = 0; j < scanlations.length; j++) {
					let chapter = this.parseChapter(scanlations[j]);
					chapter.title = title;
					chapter.chapter = chapterNum;
					chapters.push(chapter);
				}
			}
		}

		document.close();

		return chapters;
	}

	getPageList(chapterId: string): Page[] {
		let redirectRequest = Request.create(HttpMethod.GET);
		redirectRequest.url = chapterId;
		redirectRequest.headers = this.headers;
		let document = redirectRequest.html();

		let url = document.baseUri();

		if (url.includes("/paginated")) { // switch to cascade for full image list
			let request = Request.create(HttpMethod.GET);
			request.url = url.replace("/paginated", "/cascade");
			request.headers = this.headers;
			document.close();
			document = request.html();
		}

		let pageElements = document.select("div.viewer-container img").array();

		let pages: Page[] = [];

		for (let i = 0; i < pageElements.length; i++) {
			let page = new Page(i);
			if (pageElements[i].hasAttr("data-src")) {
				page.url = pageElements[i].attr("abs:data-src");
			} else {
				page.url = pageElements[i].attr("abs:src");
			}
			pages.push(page);
		}

		document.close();

		return pages;
	}

	modifyImageRequest(request: Request): void {
		request.headers = this.headers;
	}
}
