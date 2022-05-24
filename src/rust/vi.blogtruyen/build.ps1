function Package-Source {
	param (
		[Parameter(Mandatory = $true, Position = 0)]
		[String[]]$Name,
		[switch]$Build
	)
	$Name | ForEach-Object	{
		$source = $_
		if ($Build) {
			Write-Output "building $source"
			cargo +nightly build --release
		}
		Write-Output "packaging $source"
		New-Item -ItemType Directory -Path target/wasm32-unknown-unknown/release/Payload -Force | Out-Null
		Copy-Item res/* target/wasm32-unknown-unknown/release/Payload -ErrorAction SilentlyContinue
		Copy-Item sources/$source/res/* target/wasm32-unknown-unknown/release/Payload -ErrorAction SilentlyContinue
		Set-Location target/wasm32-unknown-unknown/release
		Copy-Item "$source.wasm" Payload/main.wasm
		Compress-Archive -Force -DestinationPath "../../../$source.aix" -Path Payload
		Remove-Item -Recurse -Force Payload/
		Set-Location ../../..
	}
}
Package-Source blogtruyen -Build
