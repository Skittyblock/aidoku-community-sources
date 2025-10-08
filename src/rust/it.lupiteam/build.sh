# template source build script
# usage: ./build.sh [source_name/-a]

if [ "$1" != "-a" ]; then
	# compile specified source
	cargo +nightly build --release
	
	echo "packaging $1";
	mkdir -p target/wasm32-unknown-unknown/release/Payload
	cp res/* target/wasm32-unknown-unknown/release/Payload
	cp sources/$1/res/* target/wasm32-unknown-unknown/release/Payload
	cd target/wasm32-unknown-unknown/release
	cp $1.wasm Payload/main.wasm
	zip -r $1.aix Payload
	mv $1.aix ../../../$1.aix
	rm -rf Payload
else
	# compile all sources
	cargo +nightly build --release

	for dir in sources/*/
	do
		dir=${dir%*/}
		dir=${dir##*/}
		echo "packaging $dir";

		mkdir -p target/wasm32-unknown-unknown/release/Payload
		cp res/* target/wasm32-unknown-unknown/release/Payload
		cp sources/$dir/res/* target/wasm32-unknown-unknown/release/Payload
		cd target/wasm32-unknown-unknown/release
		cp $dir.wasm Payload/main.wasm
		zip -r $dir.aix Payload >> /dev/null
		mv $dir.aix ../../../$dir.aix
		rm -rf Payload
		cd ../../../
	done
fi
