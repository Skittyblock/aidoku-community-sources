CC=clang
TARGET=wasm32
OPT=-O3
CC_FLAGS=$(OPT) --target=$(TARGET)
LD_FLAGS=-nostdlib -Wl,--no-entry -Wl,--export-dynamic #-W -Wall
SRCS=$(wildcard src/*.c) $(wildcard src/lib/*.c)
INCLUDE=-Isrc/include

build: clean package.aix

.build/main.wasm:
	@mkdir -p .build
	$(CC) $(CC_FLAGS) $(LD_FLAGS) $(INCLUDE) $(SRCS) -o $@

main.wat: .build/main.wasm
	wasm2wat $^ -o $@

package.aix: .build/main.wasm
	@mkdir -p .build/Payload
	@cp .build/main.wasm .build/Payload/main.wasm
	@cp res/* .build/Payload/
	@cd .build/ ; zip -r package.aix Payload > /dev/null
	@mv .build/package.aix package.aix

clean:
	@rm -rf main.wat package.aix .build
