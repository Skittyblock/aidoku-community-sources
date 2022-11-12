import {
  Chapter,
  Filter,
  FilterType,
  Html,
  HttpMethod,
  Manga,
  MangaPageResult,
  MangaStatus,
  Page,
  Request,
  ValueRef,
} from "aidoku-as/src";
import { json } from "aidoku-as/src/modules/net";
import { DecoderState, JSON } from "assemblyscript-json";
import { Decoder } from "./Decoder";

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
  private headers: Map<string, string>;

  constructor() {
    this.headers = new Map<string, string>();
    this.headers.set("Referer", "https://www.manhuagui.com/");
  }

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
    let chapters: Chapter[] = [];
    const elements = document.select(".chapter-list > ul > li").array();
    for (let i = 0; i < elements.length; i++) {
      let element = elements[i];
      const url = element.select("a").attr("href");
      const id = url.replace(`/comic/`, "").replace(".html", "");

      let title = element.select("a").attr("title");

      let chapter = new Chapter(id, title);
      chapter.url = `https://www.manhuagui.com${url}`;
      chapter.lang = "zh";
      chapter.chapter = (elements.length - i) as f32;

      chapters.push(chapter);
    }
    document.close();
    return chapters;
  }

  getPageList(webUrl: string, mobileUrl: string): Page[] {
    // TODO: check why cannot get the imageUrl
    let pages: Page[] = [];

    let request = Request.create(HttpMethod.GET);
    request.url = `https://m.manhuagui.com/comic/46271/664480.html`;
    request.headers = this.headers;
    let document: Html = request.html();

    let decoder = new Decoder(document.text());
    let jsonObj = decoder.decode();
    let arrOrNull = jsonObj.getArr("images")
    if (arrOrNull !== null) {
      arrOrNull._arr.forEach((value) => {
        if (value.isString) {
          let imageUrl = (<JSON.Str>value).valueOf();
          let page = new Page(0);
          page.url = encodeURI(decodeURI(imageUrl));
          pages.push(page)
        }
      })
    }

    // console.log(document.text())

    // let imageUrl = document.select("#manga > img").attr("src");

    // let page = new Page(0);
    // page.url = encodeURI(decodeURI(imageUrl));

    // pages.push(page);

    // let page = new Page(0);
    // page.url = encodeURI(decodeURI(`https://i.hamreus.com/ps4/i/5816/bouquettosshop/短篇/01.jpg.webp`));

    // pages.push(page);

    return pages;
  }

  getNumberOfPage(webUrl: string): i32 {
    let request = Request.create(HttpMethod.GET);
    request.url = webUrl;
    request.headers = this.headers;
    let document: Html = request.html();
    let pageOptions = document.select("#pageSelect > option").array();

    return pageOptions.length;
  }

  getPage(mobileUrl: string, i: i32): Page {
    let request = Request.create(HttpMethod.GET);
    request.url = `${mobileUrl}#p=${i}`;
    request.headers = this.headers;
    let document: Html = request.html();

    let imageUrl = document.select("#manga > img").attr("src");

    let page = new Page(i - 1);
    page.url = encodeURI(decodeURI(imageUrl));
    return page;
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