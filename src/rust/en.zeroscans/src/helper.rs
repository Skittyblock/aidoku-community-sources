use aidoku::std::{current_date, String};

pub fn get_date(time_ago: String) -> f64 {
	let cleaned_time_ago = String::from(time_ago.replace("ago", "").trim());

	let number = cleaned_time_ago
		.split_whitespace()
		.next()
		.unwrap_or("")
		.parse::<f64>()
		.unwrap_or(0.0);

	match cleaned_time_ago
		.to_uppercase()
		.split_whitespace()
		.last()
		.unwrap_or("")
	{
		"YEAR" | "YEARS" => current_date() - (number * 31556926.0),
		"MONTH" | "MONTHS" => current_date() - (number * 2629743.0),
		"WEEK" | "WEEKS" => current_date() - (number * 604800.0),
		"DAY" | "DAYS" => current_date() - (number * 86400.0),
		_ => current_date(),
	}
}
