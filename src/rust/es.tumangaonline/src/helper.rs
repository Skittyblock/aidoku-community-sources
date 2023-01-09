use aidoku::std::String;

pub fn string_between(string: &str, start: &str, end: &str, extension: usize) -> String {
	let start_loc = string.find(start).unwrap_or(0) + start.len();
	let half = &string[start_loc..];
	let end_loc = half.find(end).unwrap_or(half.len()) + extension;
	let result = &half[..end_loc];
	String::from(result)
}
