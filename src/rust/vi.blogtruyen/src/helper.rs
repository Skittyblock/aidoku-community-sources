use aidoku::{std::String, std::Vec, MangaStatus};
pub fn extract_f32_from_string(title: String, text: String) -> f32 {
    text.replace(&title, "")
        .chars()
        .filter(|a| (*a >= '0' && *a <= '9') || *a == ' ' || *a == '.')
        .collect::<String>()
        .split(" ")
        .collect::<Vec<&str>>()
        .into_iter()
        .map(|a| a.parse::<f32>().unwrap_or(0.0))
        .find(|a| *a > 0.0)
        .unwrap_or(0.0)
}
pub fn urlencode(string: String) -> String {
    let mut result: Vec<u8> = Vec::with_capacity(string.len() * 3);
    let hex = "0123456789ABCDEF".as_bytes();
    let bytes = string.as_bytes();

    for byte in bytes {
        let curr = *byte;
        if (b'a' <= curr && curr <= b'z')
            || (b'A' <= curr && curr <= b'Z')
            || (b'0' <= curr && curr <= b'9')
        {
            result.push(curr);
        } else {
            result.push(b'%');
            result.push(hex[curr as usize >> 4]);
            result.push(hex[curr as usize & 15]);
        }
    }
    String::from_utf8(result).unwrap_or(String::new())
}
pub fn status_from_string(status: String) -> MangaStatus {
    return match status.as_str() {
        "Đang tiến hành" => MangaStatus::Ongoing,
        "Đã hoàn thành" => MangaStatus::Completed,
        "Tạm ngưng" => MangaStatus::Hiatus,
        "Cancelled" => MangaStatus::Cancelled,
        _ => MangaStatus::Unknown,
    };
}

pub fn genre_map(genre: String) -> String {
    return String::from(match genre.as_str() {
        "16+" => "54",
        "18+" => "45",
        "Action" => "1",
        "Adult" => "2",
        "Adventure" => "3",
        "Anime" => "4",
        "Bạo lực - Máu me" => "67",
        "Comedy" => "5",
        "Comic" => "6",
        "Doujinshi" => "7",
        "Drama" => "49",
        "Ecchi" => "48",
        "Event BT" => "60",
        "Fantasy" => "50",
        "Full màu" => "64",
        "Game" => "61",
        "Gender Bender" => "51",
        "Harem" => "12",
        "Historical" => "13",
        "Horror" => "14",
        "Isekai/Dị giới/Trọng sinh" => "63",
        "Josei" => "15",
        "Live action" => "16",
        "Magic" => "46",
        "manga" => "55",
        "Manhua" => "17",
        "Manhwa" => "18",
        "Martial Arts" => "19",
        "Mature" => "20",
        "Mecha" => "21",
        "Mystery" => "22",
        "Nấu Ăn" => "56",
        "Ngôn Tình" => "65",
        "NTR" => "62",
        "One shot" => "23",
        "Psychological" => "24",
        "Romance" => "25",
        "School Life" => "26",
        "Sci-fi" => "27",
        "Seinen" => "28",
        "Shoujo" => "29",
        "Shoujo Ai" => "30",
        "Shounen" => "31",
        "Shounen Ai" => "32",
        "Slice of life" => "33",
        "Smut" => "34",
        "Soft Yaoi" => "35",
        "Soft Yuri" => "36",
        "Sports" => "37",
        "Supernatural" => "38",
        "Tạp chí truyện tranh" => "39",
        "Tragedy" => "40",
        "Trap (Crossdressing)" => "58",
        "Trinh Thám" => "57",
        "Truyện scan" => "41",
        "Tu chân - tu tiên" => "66",
        "Video Clip" => "53",
        "VnComic" => "42",
        "Webtoon" => "52",
        "Yuri" => "59",
        _ => "",
    });
}
