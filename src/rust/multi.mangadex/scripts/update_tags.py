# curl https://api.mangadex.org/manga/tag | jq "[.data[] | { type: \"genre\", name: .attributes.name.en, id: .id, canExclude: true }]"
import dataclasses
import subprocess
import json
import os
import shutil


@dataclasses.dataclass
class Tag:
    type: str
    name: str
    id: int
    canExclude: bool

    def __dict__(self):
        return dataclasses.asdict(self)


class EnhancedJSONEncoder(json.JSONEncoder):
    def default(self, o):
        if dataclasses.is_dataclass(o):
            return dataclasses.asdict(o)
        return super().default(o)


if not shutil.which("curl"):
    raise Exception("curl is not installed")

tags = json.loads(
    subprocess.check_output(["curl", "-sL", "https://api.mangadex.org/manga/tag"])
)

filters_json = os.path.join(
    os.path.dirname(os.path.realpath(__file__)), "..", "res", "filters.json"
)
with open(filters_json, "r") as f:
    filters = json.load(f)
    for filter in filters:
        name = filter.get("name")
        if name in ["Contents", "Formats", "Genres", "Themes"]:
            filter["filters"] = sorted(
                [
                    Tag("genre", tag["attributes"]["name"]["en"], tag["id"], True)
                    for tag in tags["data"]
                    if tag["attributes"]["group"] == name.lower()[:-1]
                ],
                key=lambda x: x.name.lower(),
            )

with open(filters_json, "w") as f:
    json.dump(filters, f, indent="\t", cls=EnhancedJSONEncoder)
    f.write("\n")
