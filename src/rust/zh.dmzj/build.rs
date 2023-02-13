use std::io::Result;

fn main() -> Result<()> {
	println!("cargo:rerun-if-changed=protos/comic_detail.proto");
	println!("cargo:rerun-if-changed=protos/chapter_images.proto");
	let mut config = prost_build::Config::new();
	config.btree_map(["."]);
	prost_build::compile_protos(&["protos/comic_detail.proto"], &["protos/"])?;
	prost_build::compile_protos(&["protos/chapter_images.proto"], &["protos/"])?;
	Ok(())
}
