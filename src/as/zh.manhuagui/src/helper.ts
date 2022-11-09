import {
    Chapter,
  Filter,
  FilterType,
  Html,
  Manga,
  MangaPageResult,
  MangaStatus,
  ValueRef,
} from "aidoku-as/src";

let regionOptions = [
  "all",
  "japan",
  "hongkong",
  "other",
  "europe",
  "china",
  "korea",
];

let genreOptions = [
  "ALL",
  "REXUE",
  "MAOXIAN",
  "MOHUAN",
  "SHENGUI",
  "GAOXIAO",
  "MENGXI",
  "AIQING",
  "KEHUAN",
  "MOFA",
  "GEDOU",
  "WUXIA",
  "JIZHAN",
  "ZHANZHENG",
  "JINGJI",
  "TIYU",
  "XIAOYUAN",
  "SHENGHUO",
  "LIZHI",
  "LISHI",
  "WEINIANG",
  "ZHAINAN",
  "FUNV",
  "DANMEI",
  "BAIHE",
  "HOUGONG",
  "ZHIYU",
  "MEISHI",
  "TUILI",
  "XUANYI",
  "KONGBU",
  "SIGE",
  "ZHICHANG",
  "ZHENTAN",
  "SHEHUI",
  "YINYUE",
  "WUDAO",
  "ZAZHI",
  "HEIDAO",
];

let audienceOptions = [
  "ALL",
  "SHAONV",
  "SHAONIAN",
  "QINGNIAN",
  "ERTONG",
  "TONGYONG",
];

let progressOptions = ["ALL", "LIANZAI", "WANJIE"];

export class Parser {
  parseHomePage(document: Html): MangaPageResult {
    let mangas: Manga[] = [];

    let ul = "#contList > li";
    let elements = document.select(ul).array();

    for (let i = 0; i < elements.length; i++) {
      let elem = elements[i];
      const id = elem
        .select("a")
        .attr("href")
        .replace("/comic/", "")
        .replace("/", "");
      const title = elem.select("a").attr("title");
      const img = elem.select("img").attr("src");
      let manga = new Manga(id, title);
      manga.cover_url = `https://cf.hamreus.com/cpic/b/${id}.jpg`;
      mangas.push(manga);
    }

    var hasNext = true;
    let pager = document.select("#AspNetPager1 > a").array();
    hasNext = !pager.every((p) => p.text() !== "尾页");

    document.close();
    return new MangaPageResult(mangas, hasNext);
  }

  getMangaDetails(document: Html, mangaId: string): Manga {
    const title = document.select(".book-title > h1").text().trim();
    const author = document
      .select(
        "body > div.w998.bc.cf > div.fl.w728 > div.book-cont.cf > div.book-detail.pr.fr > ul > li:nth-child(2) > span:nth-child(2) > a:nth-child(2)"
      )
      .text()
      .trim();
    const desc = document.select("#intro-cut").text().trim();
    const image = `https://cf.hamreus.com/cpic/b/${mangaId}.jpg`;
    const stat = document
      .select(
        "body > div.w998.bc.cf > div.fl.w728 > div.book-cont.cf > div.book-detail.pr.fr > ul > li.status > span > span:nth-child(0)"
      )
      .text()
      .trim();

    let manga = new Manga(mangaId, title);
    manga.author = author;
    manga.description = desc;
    manga.cover_url = image;
    manga.status = MangaStatus.Ongoing;
    manga.url = `https://www.manhuagui.com/comic/${mangaId}/`;
    if (stat == "连载中") manga.status = MangaStatus.Ongoing;
    if (stat == "已完结") manga.status = MangaStatus.Completed;

    let nsfw = false;

    let tags: string[] = [];
    const genreArr = document
      .select(
        "body > div.w998.bc.cf > div.fl.w728 > div.book-cont.cf > div.book-detail.pr.fr > ul > li:nth-child(2) > span:nth-child(1) > a"
      )
      .array();
    for (let i = 0; i < genreArr.length; i++) {
      let genre = genreArr[i].text().trim();
      tags.push(genre);
    }
    manga.categories = tags;

    document.close();
    return manga;
  }

  getChapterList(document: Html, mangaId: string): Chapter[] {
    // TODO
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
}

export class FilterMap {
  private baseUrl: string;

  constructor(baseUrl: string) {
    this.baseUrl = baseUrl;
  }

  getFilteredURL(filters: Filter[], page: number): string {
    var region: string = "all";
    var genre: string = "all";
    var audience: string = "all";
    var progress: string = "all";

    for (let i = 0; i < filters.length; i++) {
      if (filters[i].type == FilterType.Title) {
        let value = filters[i].value.toString();
        return `https://www.manhuagui.com/s/${value}_p${page}.html`;
      } else if (filters[i].name == "地区") {
        let value = filters[i].value.toInteger() as i32;
        if (value <= 0) continue;
        region = regionOptions[value].toLowerCase();
      } else if (filters[i].name == "剧情") {
        let value = filters[i].value.toInteger() as i32;
        if (value <= 0) continue;
        genre = genreOptions[value].toLowerCase();
      } else if (filters[i].name == "受众") {
        let value = filters[i].value.toInteger() as i32;
        if (value <= 0) continue;
        audience = audienceOptions[value].toLowerCase();
      } else if (filters[i].name == "进度") {
        let value = filters[i].value.toInteger() as i32;
        if (value <= 0) continue;
        progress = progressOptions[value].toLowerCase();
      }
    }
    var filterValues = [region, genre, audience, progress]
      .filter((value) => value !== "all")
      .join("_");
    if (filterValues !== "") {
      filterValues = `/${filterValues}`;
    }
    return `https://www.manhuagui.com/list${filterValues}/index_p${page
      .toString()
      .replace(".0", "")}.html`;
  }
}
