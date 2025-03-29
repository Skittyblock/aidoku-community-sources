use aidoku::{
    std::{String, Vec, ObjectRef},
    MangaStatus,
};

pub fn get_taxonomy_names(taxonomy: &ObjectRef, key: &str) -> String {
    if let Ok(array) = taxonomy.get(key).as_array() {
        let mut names = Vec::new();
        for item in array {
            if let Ok(obj) = item.as_object() {
                if let Ok(name) = obj.get("name").as_string() {
                    names.push(name.read());
                }
            }
        }
        names.join(", ")
    } else {
        String::new()
    }
}

pub fn parse_date(date_str: String) -> f64 {
    if date_str.is_empty() {
        return 0.0;
    }

    let parts: Vec<&str> = date_str.split(['T', 'Z', '-', ':'].as_ref()).collect();
    if parts.len() >= 6 {
        if let (Ok(year), Ok(month), Ok(day), Ok(hour), Ok(min), Ok(sec)) = (
            parts[0].parse::<i64>(),
            parts[1].parse::<i64>(),
            parts[2].parse::<i64>(),
            parts[3].parse::<i64>(),
            parts[4].parse::<i64>(),
            parts[5].parse::<i64>(),
        ) {
            let days_since_epoch = (year - 1970) * 365 + month * 30 + day;
            let seconds = days_since_epoch * 24 * 60 * 60 + hour * 60 * 60 + min * 60 + sec;
            return seconds as f64;
        }
    }
    0.0
}

pub fn get_status(status: i64) -> MangaStatus {
    match status {
        1 => MangaStatus::Ongoing,
        2 => MangaStatus::Completed,
        _ => MangaStatus::Unknown,
    }
}
