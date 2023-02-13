<#
.SYNOPSIS
	Template source build script for Windows
#>
#requires -version 5

$source = "nhentai_aidoku"

Write-Output "building $source"
cargo +nightly build --release

Write-Output "packaging $source"
New-Item -ItemType Directory -Path target/wasm32-unknown-unknown/release/Payload -Force | Out-Null
Copy-Item res/* target/wasm32-unknown-unknown/release/Payload -ErrorAction SilentlyContinue
Set-Location target/wasm32-unknown-unknown/release
Copy-Item "$source.wasm" Payload/main.wasm
Compress-Archive -Force -Path Payload -DestinationPath "../../../$source.aix"
Remove-Item -Recurse -Force Payload/
Set-Location ../../..
