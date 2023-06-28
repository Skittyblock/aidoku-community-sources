use aidoku::{helpers::substring::Substring, prelude::*, std::html::Node};
use alloc::{string::String, vec::Vec};

macro_rules! debug {
	($($arg:tt)*) => {{
		println!("ru.readmanga:: {}:{}: {}", file!(), line!(), format!($($arg)*))
	}};
}
pub(crate) use debug;

#[derive(Debug, Clone)]
pub struct WNode {
	repr: String,
}

impl WNode {
	pub fn new(repr: String) -> Self {
		WNode { repr }
	}

	pub fn from_node(node: Node) -> Self {
		let repr = node.outer_html().read();
		// debug!("repr: \"{}\"", repr);
		if repr.starts_with("<html>") {
			let lines: Vec<_> = repr.lines().collect();
			// debug!("lines: {:?}", lines);
			WNode {
				repr: lines[3..lines.len() - 2].join("\n"),
			}
		} else {
			WNode { repr }
		}
	}

	pub fn select(&self, selector: &str) -> Vec<WNode> {
		// we need this due to refcount error in aidoku-rs: https://github.com/Aidoku/aidoku-rs/issues/4
		let mut res = Vec::new();
		let html = self.to_node();
		for val in html.select(selector).array() {
			let node_res = val.as_node();
			if node_res.is_err() {
				debug!("failed conversion to Node");
			}
			res.push(WNode::from_node(node_res.unwrap()));
		}
		res
	}

	pub fn attr(&self, attr: &str) -> Option<String> {
		let attributes_str = self
			.repr
			.substring_before(">")?
			.substring_after("<")?
			.split_once(' ')?
			.1;

		let attr_idx = attributes_str.find(attr)?;
		let val: String = attributes_str[attr_idx..]
			.chars()
			.skip_while(|c| c != &'=')
			.skip_while(|c| c != &'"')
			.skip(1)
			.take_while(|c| c != &'"')
			.collect();

		if val.is_empty() {
			None
		} else {
			Some(val)
		}
	}

	pub fn text(&self) -> String {
		self.to_node().text().read()
	}

	pub fn data(&self) -> String {
		self.to_node().data().read()
	}

	fn to_node(&self) -> Node {
		let res = Node::new(self.repr.as_bytes());
		if res.is_err() {
			debug!("failed to create node from \"{}\"", self.repr);
		}
		res.unwrap()
	}

	pub fn to_str(&self) -> &str {
		&self.repr
	}
}
