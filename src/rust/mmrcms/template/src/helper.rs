use aidoku::{prelude::*, std::html::Node, std::String, std::Vec};

pub fn extract_f32_from_string(title: String, text: String) -> f32 {
	text.replace(&title, "")
		.replace(|a: char| a == ',' || a == '_' || a == '-', ".")
		.chars()
		.filter(|a| (*a >= '0' && *a <= '9') || *a == ' ' || *a == '.')
		.collect::<String>()
		.split(' ')
		.collect::<Vec<&str>>()
		.into_iter()
		.map(|a| a.parse::<f32>().unwrap_or(0.0))
		.find(|a| *a > 0.0)
		.unwrap_or(-1.0)
}
pub fn append_protocol(url: String) -> String {
	if !url.starts_with("http") {
		return format!("{}{}", "https:", url);
	} else {
		url
	}
}

pub fn urlencode(string: String) -> String {
	let mut result: Vec<u8> = Vec::with_capacity(string.len() * 3);
	let hex = "0123456789ABCDEF".as_bytes();
	let bytes = string.as_bytes();

	for byte in bytes {
		let curr = *byte;
		if curr.is_ascii_alphanumeric() {
			result.push(curr);
		} else {
			result.push(b'%');
			result.push(hex[curr as usize >> 4]);
			result.push(hex[curr as usize & 15]);
		}
	}
	String::from_utf8(result).unwrap_or_default()
}

pub fn text_with_newlines(node: Node) -> String {
	let html = node.html().read();
	if !String::from(html.trim()).is_empty() {
		Node::new_fragment(
			node.html()
				.read()
				.replace("<br>", "{{ .LINEBREAK }}")
				.as_bytes(),
		)
		.text()
		.read()
		.replace("{{ .LINEBREAK }}", "\n")
	} else {
		String::new()
	}
}

fn parse_email_protected<T: AsRef<str>>(data: T) -> String {
	let data = data.as_ref();
	let key = u32::from_str_radix(&data[0..2], 16).unwrap();
	let mut email = String::with_capacity(data.len() / 2);
	let mut n = 2;

	while n < data.len() {
		let chrcode = u32::from_str_radix(&data[n..n + 2], 16).unwrap() ^ key;
		email.push(char::from_u32(chrcode).unwrap_or_default());
		n += 2;
	}
	email
}

pub fn email_unprotected(node: Node) -> Node {
	let cfemail = node.select(".__cf_email__").array();
	if cfemail.is_empty() {
		node
	} else {
		let mut html = node.html().read();
		let base_uri = node.base_uri().read();
		for elem in cfemail {
			let cfnode = elem.as_node();
			let email = parse_email_protected(cfnode.attr("data-cfemail").read());
			html = html.replace(&cfnode.outer_html().read(), &email);
		}
		node.close();
		Node::new_with_uri(html, base_uri)
	}
}
