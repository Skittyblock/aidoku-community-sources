use aidoku::prelude::println;

use aidoku::{helpers::uri::QueryParameters, std::String, Filter, FilterType, MangaStatus};
use alloc::{string::ToString, vec::Vec};
extern crate alloc;

pub enum SiteId {
	MangaLib,
	HentaiLib,
	YaoiLib,
}

pub fn search(filters: Vec<Filter>) -> String {
    let mut query = QueryParameters::new();
    for filter in filters {
        match filter.kind {
            FilterType::Title => {
                if let Ok(x) = filter.value.as_string() {
                    query.push("q", Some(&x.read()));
                }
            }
            FilterType::Check => {
                match filter.name.as_str() {
                    // Status
                    "Онгоинг" => {
                        query.push("status[]", Some("1"));
                    },
                    "Завершён" => {
                        query.push("status[]", Some("2"));
                    },
                    "Анонс" => {
                        query.push("status[]", Some("3"));
                    },
                    "Приостановлен" => {
                        query.push("status[]", Some("4"));
                    },
                    "Выпуск прекращён" => {
                        query.push("status[]", Some("5"));
                    },

                    // Type of manga

                    "Манга" => {
                        query.push("types[]", Some("1"));
                    },
                    "Манхва" => {
                        query.push("types[]", Some("5"));
                    },
                    "Руманга" => {
                        query.push("types[]", Some("8"));
                    },
                    "OEL-манга" => {
                        query.push("types[]", Some("4"));
                    },
                    "Маньхуа" => {
                        query.push("types[]", Some("6"));
                    },
                    "Комикс" => {
                        query.push("types[]", Some("9"));
                    }
                    _ => continue,
                }
            }
            FilterType::Sort => {
                if let Ok(value) = filter.value.as_object() {
                    let index = value.get("index").as_int().unwrap_or(0);
                    let asc = value.get("ascending").as_bool().unwrap_or(false);
                    match index {
                        0 => {
                            query.push("fields[]", Some("rate"))
                        },
                        1 => {
                            query.push("rate_min", Some("50")); // Idk. It's was at the XHR request
                            query.push("sort_by", Some("rate_avg"))
                        },
                        2 => {
                            query.push("sort_by", Some("views"))
                        },
                        3 => {
                            query.push("sort_by", Some("chap_count"))
                        },
                        4 => {
                            query.push("sort_by", Some("last_chapter_at"))
                        },
                        5 => {
                            query.push("sort_by", Some("created_at"))
                        },
                        6 => {
                            query.push("sort_by", Some("name"))
                        },
                        7 => {
                            query.push("sort_by", Some("rus_name"))
                        },
                        _ => continue
                    }
                    if asc {
                        query.push("sort_type", Some("asc"))
                    }
                }
            }
            _ => continue,
        }
    }

    query.to_string()
}

pub fn id_to_status(id: i64) -> MangaStatus {
    match id {
        1 => MangaStatus::Ongoing,
        2 => MangaStatus::Completed,
        4 => MangaStatus::Hiatus,
        5 => MangaStatus::Cancelled,
        _ => MangaStatus::Unknown,
    }
}

pub fn siteid_to_domain(site: &SiteId) -> String {
    match site {
        SiteId::MangaLib => String::from("mangalib.me"),
        SiteId::YaoiLib =>  String::from("slashlib.me"),
        SiteId::HentaiLib => String::from("hentailib.me")
    }
}

pub fn route(site: &SiteId) -> String {
	match site {
		SiteId::MangaLib => String::from("1"),
		SiteId::HentaiLib => String::from("4"),
		SiteId::YaoiLib => String::from("2"),
	}
}