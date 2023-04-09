#!/bin/sh -e

# source filters generator script
# usage: ./scripts/filters.sh

json='[
	{type: "title"},
	{type: "author"},
	{
		type: "select",
		name: "Status",
		options: ["Any", "Completed", "Ongoing", "On Hiatus", "Canceled"]
	},
	{
		type: "sort",
		name: "Sort by",
		canAscend: true,
		options: ["Title", "Views", "Latest upload", "Chapter count"],
		default: {index: 0, ascending: true}
	},
	{
		type: "group",
		name: "Categories",
		filters: .results | map({type: "genre", name})
	}
]'

for dir in "${0%/*}"/../sources/*; do
	url="$(jq -r .info.url -- "$dir/res/source.json")"
	curl -Ssf "$url/api/v2/categories" | \
		jq --tab -r "$json" > "$dir/res/filters.json"
done
