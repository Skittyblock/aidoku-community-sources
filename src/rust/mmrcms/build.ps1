<#
.SYNOPSIS
	Template source build script for Windows
#>
#requires -version 5
[cmdletbinding()]
param (
	[Parameter(ParameterSetName="help", Mandatory)]
	[alias('h')]
	[switch]$help,

	[Parameter(ParameterSetName="all", Mandatory)]
	[alias('a')]
	[switch]$all,

	[Parameter(Position=0, ParameterSetName="some", Mandatory)]
	[alias('s')]
	[string[]]$sources
)

function Package-Source {
	param (
		[Parameter(Mandatory = $true, Position = 0)]
		[String[]]$Name,

		[switch]$Build
	)
	$Name | ForEach-Object  {
		$source = $_
		if ($Build) {
			Write-Output "building $source"
			Set-Location ./sources/$source
			cargo +nightly build --release
			Set-Location ../..
		}

		Write-Output "packaging $source"
		New-Item -ItemType Directory -Path target/wasm32-unknown-unknown/release/Payload -Force | Out-Null
		Copy-Item res/* target/wasm32-unknown-unknown/release/Payload -ErrorAction SilentlyContinue
		Copy-Item sources/$source/res/* target/wasm32-unknown-unknown/release/Payload -ErrorAction SilentlyContinue
		Set-Location target/wasm32-unknown-unknown/release
		Copy-Item "$source.wasm" Payload/main.wasm
		Compress-Archive -Force -Path Payload -DestinationPath "../../../$source.aix"
		Remove-Item -Recurse -Force Payload/
		Set-Location ../../..
	}
}

if ($help -or ($null -eq $PSBoundParameters.Keys)) {
	Get-Help $MyInvocation.MyCommand.Path -Detailed
	break
}

if ($all) {
	cargo +nightly build --release
	Get-ChildItem ./sources | ForEach-Object {
		$source = (Split-Path -Leaf $_)
		Package-Source $source
	}
} else {
	$sources | ForEach-Object {
		Package-Source $_ -Build
	}
}
