use aidoku::std::String;

pub fn remove_special_chars(string: String) -> String {
	let chars_to_replace = ['[', '!', '#', ':', '(', ')', ']'];
	let mut result = String::new();

	for c in string.chars() {
		if chars_to_replace.contains(&c) {
			result.push(' ');
		} else {
			result.push(c);
		}
	}

	result
}
