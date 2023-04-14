use aidoku::{
    error::Result,
    helpers::uri::encode_uri,
    prelude::{format, println},
    std::{html::Node, String, Vec},
    Chapter, Filter, FilterType, Manga, MangaContentRating, MangaPageResult, MangaStatus,
    MangaViewer, Page,
};

const BASE_URL: &str = "https://www.baozimh.com";

const CLASSIFY_REGION: [&str; 5] = ["all", "cn", "jp", "kr", "en"];
const CLASSIFY_STATE: [&str; 3] = ["all", "serial", "pub"];
const CLASSIFY_TYPE: [&str; 26] = [
    "all",
    "lianai",
    "chunai",
    "gufeng",
    "yineng",
    "xuanyi",
    "juqing",
    "kehuan",
    "qihuan",
    "xuanhuan",
    "chuanyue",
    "mouxian",
    "tuili",
    "wuxia",
    "gedou",
    "zhanzheng",
    "rexie",
    "gaoxiao",
    "danuzhu",
    "dushi",
    "zongcai",
    "hougong",
    "richang",
    "hanman",
    "shaonian",
    "qita",
];
const CLASSIFY_FILTER: [&str; 9] = [
    "%2a",
    "ABCD",
    "EFGH",
    "IJKL",
    "MNOP",
    "QRST",
    "UVW",
    "XYZ",
    "0123456789",
];

pub fn get_filtered_url(filters: Vec<Filter>, page: i32, url: &mut String) {
    let mut is_searching = false;
    let mut search_str = String::new();
    url.push_str(BASE_URL);

    let mut c_region: &str = "all";
    let mut c_state: &str = "all";
    let mut c_type: &str = "all";
    let mut c_filter: &str = "%2a";

    for filter in filters {
        match filter.kind {
            FilterType::Title => {
                if let Ok(filter_value) = filter.value.as_string() {
                    search_str.push_str(encode_uri(&filter_value.read()).as_str());
                    is_searching = true;
                }
            }
            FilterType::Select => {
                let index = filter.value.as_int().unwrap_or(0) as usize;
                match filter.name.as_str() {
                    "地區" => c_region = CLASSIFY_REGION[index],
                    "連載情況" => c_state = CLASSIFY_STATE[index],
                    "類型" => c_type = CLASSIFY_TYPE[index],
                    "依字母篩選" => c_filter = CLASSIFY_FILTER[index],
                    _ => continue,
                };
            }
            _ => continue,
        }
    }

    if is_searching {
        url.push_str(format!("/search?q={}", search_str).as_str());
    } else {
        url.push_str(
            format!(
                "/classify?type={}&region={}&state={}&filter={}",
                c_type, c_region, c_state, c_filter
            )
            .as_str(),
        );
    }
}

pub fn parse_home_page(html: Node) -> Result<MangaPageResult> {
    todo!()
}

pub fn parse_search_page(html: Node) -> Result<MangaPageResult> {
    todo!()
}

pub fn get_manga_details(html: Node, manga_id: String) -> Result<Manga> {
    let cover = format!("https://static-tw.baozimh.com/cover/{}.jpg", manga_id);
    // let title = html.select(".comics-detail__title").text().read();
    let title = html
        .select("meta[name='og:novel:book_name']")
        .attr("content")
        .read();
    // let author = html.select(".comics-detail__author").text().read();
    let author = html
        .select("meta[name='og:novel:author']")
        .attr("content")
        .read();
    let description = html.select(".comics-detail__desc").text().read();
    let url = format!("{}/comic/{}", BASE_URL, manga_id);
    let categories_array = html.select(".tag-list").select("span").array();
    let status_str = html
        .select("meta[name='og:novel:status']")
        .attr("content")
        .read();

    let mut categories: Vec<String> = Vec::new();
    for category in categories_array {
        let category_str = category.as_node().expect("node array").text().read();
        if category_str != "連載中" || category_str != "已完結" || category_str.is_empty() {
            categories.push(category_str);
        }
    }

    let status = if status_str.contains("連載中") {
        MangaStatus::Ongoing
    } else if status_str.contains("已完結") {
        MangaStatus::Completed
    } else {
        MangaStatus::Unknown
    };

    let manga = Manga {
        id: manga_id,
        cover,
        title,
        author: author.clone(),
        artist: author,
        description,
        url,
        categories,
        status,
        nsfw: MangaContentRating::Safe,
        viewer: MangaViewer::Scroll,
    };

    Ok(manga)
}

pub fn get_chapter_list(html: Node) -> Result<Vec<Chapter>> {
    let mut chapters: Vec<Chapter> = Vec::new();

    todo!();

    Ok(chapters)
}

pub fn get_page_list(html: Node) -> Result<Vec<Page>> {
    let mut pages: Vec<Page> = Vec::new();

    todo!();

    Ok(pages)
}
