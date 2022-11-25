use aidoku::std::String;

// get string after and before substrings, with an added character count
// extension
pub fn string_between(string: &str, start: &str, end: &str, extension: usize) -> String {
	let start_loc = string.find(start).unwrap_or(0) + start.len();
	let half = &string[start_loc..];
	let end_loc = half.find(end).unwrap_or(string.len()) + extension;
	let result = &half[..end_loc];
	String::from(result)
}

// converts chapter id to a (optionally) padded float string
// e.g. "100345" -> "0034.5"
pub fn chapter_image(id: &str, pad: bool) -> String {
	let mut new_str = String::new();

	if pad {
		new_str.push_str(&id[1..id.len() - 1]);
	} else {
		new_str.push_str(id[1..id.len() - 1].trim_start_matches('0'));
	}

	if new_str.is_empty() {
		new_str.push('0');
	}

	if id.chars().last().unwrap_or('0') != '0' {
		new_str.push('.');
		new_str.push_str(id.chars().last().unwrap_or('0').encode_utf8(&mut [0; 1]));
	}

	new_str
}

// takes a chapter id and returns the url path suffix
// e.g "101280" -> "-chapter-128.html"
//     "500480" -> "-chapter-48-index-5.html"
//     "912345" -> "-chapter-1234.5-index-9.html"
pub fn chapter_url_encode(id: &str) -> String {
	let mut output = String::from("-chapter-");

	output.push_str(&chapter_image(id, false));

	let index = id.chars().next().unwrap_or('1');
	if index != '1' {
		output.push_str("-index-");
		output.push_str(index.encode_utf8(&mut [0; 1]));
	}

	output.push_str(".html");

	output
}
