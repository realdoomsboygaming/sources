# curl https://api.mangadex.org/manga/tag | jq "[.data[] | { type: \"genre\", name: .attributes.name.en, id: .id, canExclude: true }]"
import dataclasses
import subprocess
import json
import os
import shutil

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
        name = filter.get("title")
        if name in ["Content", "Format", "Genre", "Theme"]:
            items = sorted(
                [
                    (tag["attributes"]["name"]["en"], tag["id"])
                    for tag in tags["data"]
                    if tag["attributes"]["group"] == name.lower()
                ],
                key=lambda x: x[0].lower(),
            )
            filter["options"] = [item[0] for item in items]
            filter["ids"] = [item[1] for item in items]


with open(filters_json, "w") as f:
    json.dump(filters, f, indent="\t", cls=EnhancedJSONEncoder)
    f.write("\n")
