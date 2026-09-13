#!/bin/zsh
set -euo pipefail
cd "$(dirname "$0")/.."

xcodegen generate
xcodebuild -project HerdrBell.xcodeproj -scheme HerdrBell -configuration Release build

APP=$(find ~/Library/Developer/Xcode/DerivedData -name HerdrBell.app -path "*Release*" | head -1)
if [ -z "$APP" ]; then
    echo "Release build not found" >&2
    exit 1
fi

rm -rf build
mkdir -p build
cp -R "$APP" build/HerdrBell.app
codesign --force --deep --sign - build/HerdrBell.app

(cd build && zip -qry HerdrBell.zip HerdrBell.app)
echo "Packaged: build/HerdrBell.app and build/HerdrBell.zip"
