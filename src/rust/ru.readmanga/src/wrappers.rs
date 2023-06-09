use aidoku::{prelude::*, std::html::Node};
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
	pub fn new(node: Node) -> Self {
		WNode {
			repr: node.outer_html().read(),
		}
	}

	pub fn select<T: AsRef<str>>(&self, selector: T) -> Vec<WNode> {
		// we need this due to refcount error in aidoku-rs: https://github.com/Aidoku/aidoku-rs/issues/4
		let mut res = Vec::new();
		let html = self.to_node();
		for val in html.select(selector).array() {
			let node_res = val.as_node();
			if node_res.is_err() {
				debug!("failed conversion to Node");
			}
			res.push(WNode::new(node_res.unwrap()));
		}
		res
	}

	fn to_node(&self) -> Node {
		let res = Node::new(self.repr.as_bytes());
		if res.is_err() {
			debug!("failed to create node from \"{}\"", self.repr);
		}
		res.unwrap()
	}
}
