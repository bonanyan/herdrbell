#!/bin/zsh
# Imports hand-drawn SVG glyphs from Sources/Resources/Artwork/ into the
# asset catalog as template imagesets (StatusIcons/<name>.imageset).
#
# A glyph is imported only when it actually contains artwork: the
# "guides-delete-me" layer is stripped first, and at least one drawable
# shape must remain. Empty templates are left alone, and the app keeps
# rendering the scheme's SF Symbol fallbacks until then.
set -euo pipefail
cd "$(dirname "$0")/.."

python3 - <<'PY'
import json
import shutil
import sys
import xml.etree.ElementTree as ET
from pathlib import Path

ARTWORK = Path("Sources/Resources/Artwork")
TARGET = Path("Sources/Resources/Assets.xcassets/StatusIcons")
GLYPHS = [
    "status-blocked",
    "status-working",
    "status-done",
    "status-idle",
    "status-unknown",
    "aggregate-idle",
    "aggregate-disconnected",
    "marker-focused",
]
SVG = "{http://www.w3.org/2000/svg}"
DRAWABLE = {"path", "rect", "circle", "ellipse", "line", "polyline", "polygon", "use", "text", "image"}

def strip_guides(root):
    removed = 0
    for parent in root.iter():
        for child in list(parent):
            if child.tag == f"{SVG}g" and (child.get("id") or "").startswith("guides"):
                parent.remove(child)
                removed += 1
    return removed

def ensure_intrinsic_size(root):
    # Illustrator exports viewBox-only SVGs; without width/height the
    # compiled asset has zero intrinsic size and renders invisible.
    if root.get("width") and root.get("height"):
        return False
    viewBox = (root.get("viewBox") or "").split()
    if len(viewBox) != 4:
        return False
    root.set("width", viewBox[2])
    root.set("height", viewBox[3])
    return True

def has_artwork(root):
    for el in root.iter():
        tag = el.tag.replace(SVG, "")
        if tag in DRAWABLE:
            return True
    return False

TARGET.mkdir(parents=True, exist_ok=True)
imported, skipped = [], []
for name in GLYPHS:
    src = ARTWORK / f"{name}.svg"
    if not src.exists():
        skipped.append((name, "no file in Artwork/"))
        continue
    tree = ET.parse(src)
    root = tree.getroot()
    strip_guides(root)
    if not has_artwork(root):
        skipped.append((name, "still empty (guides only)"))
        continue
    sized = ensure_intrinsic_size(root)
    imageset = TARGET / f"{name}.imageset"
    imageset.mkdir(parents=True, exist_ok=True)
    tree.write(imageset / f"{name}.svg", xml_declaration=True, encoding="UTF-8")
    contents = {
        "images": [{"filename": f"{name}.svg", "idiom": "universal"}],
        "info": {"author": "xcode", "version": 1},
        "properties": {
            "preserves-vector-representation": True,
            "template-rendering-intent": "template",
        },
    }
    (imageset / "Contents.json").write_text(json.dumps(contents, indent=2) + "\n")
    imported.append(name)

for name, reason in skipped:
    print(f"skip    {name}: {reason}")
for name in imported:
    print(f"import  {name}")
if not imported:
    print("Nothing imported yet — Custom scheme keeps SF Symbol fallbacks.")
PY

if [ -f Sources/Resources/Artwork/app-icon-master.png ]; then
    echo "app icon master found — slicing sizes..."
    swift Scripts/MakeIcon.swift --master Sources/Resources/Artwork/app-icon-master.png
else
    echo "note: drop a 1024x1024 app-icon-master.png into Artwork/ to rebuild the app icon."
fi
