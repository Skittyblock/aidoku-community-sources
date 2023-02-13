use std::io::Result;

fn main() -> Result<()> {
	println!("cargo:rerun-if-changed=protos/detail_response.proto");
	let mut config = prost_build::Config::new();
	config.btree_map(["."]);
	prost_build::compile_protos(&["protos/detail_response.proto"], &["protos/"])?;
	Ok(())
}
