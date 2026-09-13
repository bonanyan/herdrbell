#!/bin/zsh
set -euo pipefail
cd "$(dirname "$0")/.."

if [ ! -d build/HerdrBell.app ]; then
    ./Scripts/package.sh
fi

pkill -f "HerdrBell.app/Contents/MacOS/HerdrBell" 2>/dev/null || true
sleep 1
rm -rf /Applications/HerdrBell.app
cp -R build/HerdrBell.app /Applications/
echo "Installed to /Applications/HerdrBell.app"
open /Applications/HerdrBell.app
