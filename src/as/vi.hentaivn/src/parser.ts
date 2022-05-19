import {
  Chapter,
  defaults,
  Filter,
  FilterType,
  Html,
  Manga,
  MangaContentRating,
  MangaPageResult,
  MangaStatus,
  MangaViewer,
  Page,
  ValueRef,
} from "aidoku-as/src/index";
import { Constants } from "./constants";

// MARK: Utilities
function parseEmailProtected(data: string): string {
  let email = "";
  const key = parseInt(data.substr(0, 2), 16) as i64;
  for (let n = 2; data.length - n; n += 2) {
    const char = <i64> parseInt(data.substr(n, 2), 16) ^ key;
    email += String.fromCharCode(<i32> char);
  }
  return email;
}

function parseAllEmailProtected(document: Html): Html {
  let html = document.html();
  const elements = document.select("span.__cf_email__").array();
  for (let i = 0; i < elements.length; i++) {
    const elem = elements[i];
    const email = parseEmailProtected(elem.attr("data-cfemail"));
    html = html.replace(elem.outerHtml(), email);
  }

  document.close();
  return Html.parse(String.UTF8.encode(html));
}

function transformQuality(url: string, quality: i64): string {
  if (quality == 1200) {
    return url;
  }

  // https://upv2.hentaivn.moe/images/800/2022/05/11/1652291962-pic01.jpg?width=1200
  // ['https:', '', 'upv2.hentaivn.moe', 'images', '800', '2022', '05', '11', '1652291962-pic01.jpg?width=1200']
  let urlSplit = url.split("/");
  const width = min(quality + 400, 9999);
  urlSplit[4] = urlSplit[4].replace(
    "1200",
    quality.toString().replaceAll(".0", ""),
  );
  urlSplit[8] = urlSplit[8].replace(
    "?imgmax=1200",
    `?width=${width.toString().replaceAll(".0", "")}`,
  );
  return urlSplit.join("/");
}

// MARK: Parser
export class Parser {
  parseNewOrCompletePage(document: Html): MangaPageResult {
    const elements = document.select("li.item").array();
    const results = elements.map<Manga>((elem) => {
      const id = elem.select('div.box-description > p > a[href*="doc-truyen"]')
        .attr("href").split("/").pop();
      const title = parseAllEmailProtected(elem.select(
        'div.box-description > p > a[href*="doc-truyen"]',
      )).text().trim();
      const img = elem.select("div.box-cover > a > img").attr("data-src");
      const tags = elem.select("div.box-description > p:contains(Thể Loại)")

      const manga = new Manga(id, title);
      manga.cover_url = img;
      manga.rating = tags.text().includes("Non-hen")
        ? MangaContentRating.Suggestive
        : MangaContentRating.NSFW;
      return manga;
    });

    document.close();
    return new MangaPageResult(results, results.length !== 0);
  }

  parseSearchPage(document: Html, page: i32, includesNonHen: bool): MangaPageResult {
    const results: Manga[] = [];
    const elements = document.select("li.search-li").array();
    for (let i = 0; i < elements.length; i++) {
      const elem = elements[i];
      const id = elem.select("div.search-des > a").attr("href").split("/")
        .pop();
      const title = parseAllEmailProtected(
        elem.select("div.search-des > a > b")
      ).text().trim();
      
      const img = elem.select("div.search-img > a > img").attr("src");

      const manga = new Manga(id, title);
      manga.cover_url = img;
      manga.rating = includesNonHen
        ? MangaContentRating.Suggestive
        : MangaContentRating.NSFW;

      results.push(manga);
    }

    const lastPage = <i32> parseInt(
      document.select("ul.pagination > li").array().pop().text().trim(),
    );
    const searchHasNextPage = lastPage !== page;
    const mangaPageResult = new MangaPageResult(results, searchHasNextPage);
    document.close();
    return mangaPageResult;
  }

  getMangaDetails(document: Html, mangaId: string): Manga {
    const title = parseAllEmailProtected(
      document.select('div.page-info > h1[itemprop="name"] > a'),
    )
      .text()
      .trim()
      .split(" - ")
      .shift();
    const author = document.select('span.info ~ span > a[href*="tacgia"]')
      .array()
      .map<string>((elem) => elem.text().trim())
      .join(", ");
    const desc = document.select("p:contains(Nội dung) + p")
      .text().trim();
    const img = document.select("div.page-ava > img").attr("src");
    const stat = document.select("span.info:contains(Tình Trạng) + span > a")
      .text().trim();
    const tags = document.select("a.tag")
      .array()
      .map<string>((elem) => elem.text().trim());

    const manga = new Manga(mangaId, title);
    manga.author = author;
    manga.description = desc;
    manga.cover_url = img;
    manga.status = stat == "Đã hoàn thành"
      ? MangaStatus.Completed
      : MangaStatus.Ongoing;
    manga.categories = tags;
    manga.rating = tags.includes("Non-hen") 
      ? MangaContentRating.Suggestive 
      : MangaContentRating.NSFW;
    manga.viewer = tags.includes("Webtoon")
      ? MangaViewer.Scroll
      : MangaViewer.RTL;
    manga.url = `${Constants.domain}/${mangaId}`;

    document.close();
    return manga;
  }

  getChapterList(document: Html): Chapter[] {
    const elements = document.select("table.listing > tbody > tr").array();

    const chapters = elements.map<Chapter>((elem) => {
      const url = elem.select("td:nth-child(1) > a").attr("href");
      const chapterInfoStr = elem.select("td:nth-child(1) > a > h2").text();
      const chapterInfo = chapterInfoStr.includes(":")
        ? chapterInfoStr.split(": ")
        : chapterInfoStr.split(" - ");

      const chapterDateObject = ValueRef.string(
        elem.select("td:nth-child(2)").text(),
      );
      const chapterDate = chapterDateObject.toDate(
        "dd/MM/yyyy",
        "en_US",
        "Asia/Ho_Chi_Minh",
      );
      chapterDateObject.close();

      const chapter = new Chapter(
        `${Constants.domain}${url}`,
        chapterInfo.length > 1
          ? chapterInfo[1]
          : chapterInfo[0].includes("Chap")
          ? ""
          : chapterInfo[0],
      );
      chapter.url = `${Constants.domain}${url}`;
      chapter.lang = "vi";
      chapter.chapter = chapterInfo[0].includes("Chap")
        ? parseFloat(chapterInfo[0].split(" ").pop()) as f32
        : 1;
      chapter.dateUpdated = chapterDate;

      return chapter;
    });

    document.close();
    return chapters;
  }

  getPageList(document: Html): Page[] {
    const elements = document.select("div#image > img").array();

    if (elements.length == 0) {
      const tmp = new Page(0);
      tmp.url = Constants.imageNotFoundURL;
      document.close();
      return [tmp];
    }

    const pages = elements.map<Page>((elem: Html, idx: i32) => {
      const page = new Page(idx);
      const pageQuality = defaults.get(Constants.pageQualityKey).toInteger();
      let url = elem.attr("src");
      page.url = transformQuality(url, pageQuality);
      return page;
    });

    document.close();
    return pages;
  }
}

export class Search {
  private baseUrl: string;
  private genreMap: Map<string, string> = new Map<string, string>();

  constructor(baseUrl: string) {
    this.baseUrl = baseUrl;
    this.genreMap.set("3D Hentai", "3");
    this.genreMap.set("Action", "5");
    this.genreMap.set("Adult", "116");
    this.genreMap.set("Adventure", "203");
    this.genreMap.set("Ahegao", "20");
    this.genreMap.set("Anal", "21");
    this.genreMap.set("Angel", "249");
    this.genreMap.set("Ảnh động", "131");
    this.genreMap.set("Animal", "127");
    this.genreMap.set("Animal girl", "22");
    this.genreMap.set("Áo Dài", "279");
    this.genreMap.set("Apron", "277");
    this.genreMap.set("Artist CG", "115");
    this.genreMap.set("Based Game", "130");
    this.genreMap.set("BBM", "257");
    this.genreMap.set("BBW", "251");
    this.genreMap.set("BDSM", "24");
    this.genreMap.set("Bestiality", "25");
    this.genreMap.set("Big Ass", "133");
    this.genreMap.set("Big Boobs", "23");
    this.genreMap.set("Big Penis", "32");
    this.genreMap.set("Blackmail", "267");
    this.genreMap.set("Bloomers", "27");
    this.genreMap.set("BlowJobs", "28");
    this.genreMap.set("Body Swap", "29");
    this.genreMap.set("Bodysuit", "30");
    this.genreMap.set("Bondage", "254");
    this.genreMap.set("Breast Sucking", "33");
    this.genreMap.set("BreastJobs", "248");
    this.genreMap.set("Brocon", "31");
    this.genreMap.set("Brother", "242");
    this.genreMap.set("Business Suit", "241");
    this.genreMap.set("Catgirls", "39");
    this.genreMap.set("Che ít", "101");
    this.genreMap.set("Che nhiều", "129");
    this.genreMap.set("Cheating", "34");
    this.genreMap.set("Chikan", "35");
    this.genreMap.set("Chinese Dress", "271");
    this.genreMap.set("Có che", "100");
    this.genreMap.set("Comedy", "36");
    this.genreMap.set("Comic", "120");
    this.genreMap.set("Condom", "210");
    this.genreMap.set("Cosplay", "38");
    this.genreMap.set("Cousin", "2");
    this.genreMap.set("Crotch Tattoo", "275");
    this.genreMap.set("Cunnilingus", "269");
    this.genreMap.set("Dark Skin", "40");
    this.genreMap.set("Daughter", "262");
    this.genreMap.set("Deepthroat", "268");
    this.genreMap.set("Demon", "132");
    this.genreMap.set("DemonGirl", "212");
    this.genreMap.set("Devil", "104");
    this.genreMap.set("DevilGirl", "105");
    this.genreMap.set("Dirty", "253");
    this.genreMap.set("Dirty Old Man", "41");
    this.genreMap.set("DogGirl", "260");
    this.genreMap.set("Double Penetration", "42");
    this.genreMap.set("Doujinshi", "44");
    this.genreMap.set("Drama", "4");
    this.genreMap.set("Drug", "43");
    this.genreMap.set("Ecchi", "45");
    this.genreMap.set("Elder Sister", "245");
    this.genreMap.set("Elf", "125");
    this.genreMap.set("Exhibitionism", "46");
    this.genreMap.set("Fantasy", "123");
    this.genreMap.set("Father", "243");
    this.genreMap.set("Femdom", "47");
    this.genreMap.set("Fingering", "48");
    this.genreMap.set("Footjob", "108");
    this.genreMap.set("Foxgirls", "259");
    this.genreMap.set("Full Color", "37");
    this.genreMap.set("Furry", "202");
    this.genreMap.set("Futanari", "50");
    this.genreMap.set("GangBang", "51");
    this.genreMap.set("Garter Belts", "206");
    this.genreMap.set("Gender Bender", "52");
    this.genreMap.set("Ghost", "106");
    this.genreMap.set("Glasses", "56");
    this.genreMap.set("Gothic Lolita", "264");
    this.genreMap.set("Group", "53");
    this.genreMap.set("Guro", "55");
    this.genreMap.set("Hairy", "247");
    this.genreMap.set("Handjob", "57");
    this.genreMap.set("Harem", "58");
    this.genreMap.set("HentaiVN", "102");
    this.genreMap.set("Historical", "80");
    this.genreMap.set("Horror", "122");
    this.genreMap.set("Housewife", "59");
    this.genreMap.set("Humiliation", "60");
    this.genreMap.set("Idol", "61");
    this.genreMap.set("Imouto", "244");
    this.genreMap.set("Incest", "62");
    this.genreMap.set("Insect (Côn Trùng)", "26");
    this.genreMap.set("Isekai", "280");
    this.genreMap.set("Không che", "99");
    this.genreMap.set("Kimono", "110");
    this.genreMap.set("Kuudere", "265");
    this.genreMap.set("Lolicon", "63");
    this.genreMap.set("Maids", "64");
    this.genreMap.set("Manhua", "273");
    this.genreMap.set("Manhwa", "114");
    this.genreMap.set("Masturbation", "65");
    this.genreMap.set("Mature", "119");
    this.genreMap.set("Miko", "124");
    this.genreMap.set("Milf", "126");
    this.genreMap.set("Mind Break", "121");
    this.genreMap.set("Mind Control", "113");
    this.genreMap.set("Mizugi", "263");
    this.genreMap.set("Monster", "66");
    this.genreMap.set("Monstergirl", "67");
    this.genreMap.set("Mother", "103");
    this.genreMap.set("Nakadashi", "205");
    this.genreMap.set("Netori", "1");
    this.genreMap.set("Non-hen", "201");
    this.genreMap.set("NTR", "68");
    this.genreMap.set("Nun", "272");
    this.genreMap.set("Nurse", "69");
    this.genreMap.set("Old Man", "211");
    this.genreMap.set("Oneshot", "71");
    this.genreMap.set("Oral", "70");
    this.genreMap.set("Osananajimi", "209");
    this.genreMap.set("Paizuri", "72");
    this.genreMap.set("Pantyhose", "204");
    this.genreMap.set("Ponytail", "276");
    this.genreMap.set("Pregnant", "73");
    this.genreMap.set("Rape", "98");
    this.genreMap.set("Rimjob", "258");
    this.genreMap.set("Romance", "117");
    this.genreMap.set("Ryona", "207");
    this.genreMap.set("Scat", "134");
    this.genreMap.set("School Uniform", "74");
    this.genreMap.set("SchoolGirl", "75");
    this.genreMap.set("Series", "87");
    this.genreMap.set("Sex Toys", "88");
    this.genreMap.set("Shimapan", "246");
    this.genreMap.set("Short Hentai", "118");
    this.genreMap.set("Shota", "77");
    this.genreMap.set("Shoujo", "76");
    this.genreMap.set("Siscon", "79");
    this.genreMap.set("Sister", "78");
    this.genreMap.set("Slave", "82");
    this.genreMap.set("Sleeping", "213");
    this.genreMap.set("Small Boobs", "84");
    this.genreMap.set("Son", "278");
    this.genreMap.set("Sports", "83");
    this.genreMap.set("Stockings", "81");
    this.genreMap.set("Supernatural", "85");
    this.genreMap.set("Sweating", "250");
    this.genreMap.set("Swimsuit", "86");
    this.genreMap.set("Tall Girl", "266");
    this.genreMap.set("Teacher", "91");
    this.genreMap.set("Tentacles", "89");
    this.genreMap.set("Time Stop", "109");
    this.genreMap.set("Tomboy", "90");
    this.genreMap.set("Tracksuit", "252");
    this.genreMap.set("Transformation", "256");
    this.genreMap.set("Trap", "92");
    this.genreMap.set("Truyện Việt", "274");
    this.genreMap.set("Tsundere", "111");
    this.genreMap.set("Twins", "93");
    this.genreMap.set("Twintails", "261");
    this.genreMap.set("Vampire", "107");
    this.genreMap.set("Vanilla", "208");
    this.genreMap.set("Virgin", "95");
    this.genreMap.set("Webtoon", "270");
    this.genreMap.set("X-ray", "94");
    this.genreMap.set("Yandere", "112");
    this.genreMap.set("Yaoi", "96");
    this.genreMap.set("Yuri", "97");
    this.genreMap.set("Zombie", "128");
  }

  private getGenres(name: string): string {
    return this.genreMap.get(name);
  }

  getFilteredURL(filters: Filter[], page: i32): string | null {
    const qs: string[] = [
      "dou=",
      "char=",
      `page=${page.toString().replaceAll(".0", "")}`,
      "search=",
    ];
    if (filters.length == 0) {
      qs.push(`name=`);
    }
    for (let i = 0; i < filters.length; i++) {
      const filter = filters[i];
      if (filter.type == FilterType.Title) {
        qs.push(
          `name=${encodeURI(filter.value.toString().replaceAll(" ", "+"))}`,
        );
      }
      if (filter.type == FilterType.Genre && filter.value.toInteger() == 1) {
        qs.push(`tag%5B%5D=${this.getGenres(filter.name)}`);
      }
    }
    return `${this.baseUrl}/forum/search-plus.php?${qs.join("&")}`;
  }
}
