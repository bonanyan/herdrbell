APP        := HerdrBell
SCHEME     := HerdrBell
CONFIG     ?= Release
BUILD_DIR  := build
DERIVED    := $(BUILD_DIR)/DerivedData
BUILT_APP  := $(DERIVED)/Build/Products/$(CONFIG)/$(APP).app

MASTER  := Sources/Resources/Artwork/app-icon-master.png
ICONSET := Sources/Resources/Assets.xcassets/AppIcon.appiconset

SOURCES := $(shell find Sources Tests project.yml -type f 2>/dev/null)

.DEFAULT_GOAL := all

.PHONY: all package test install clean icons

ZIP   := $(BUILD_DIR)/$(APP).zip
DMG   := $(BUILD_DIR)/$(APP).dmg
STAGE := $(BUILD_DIR)/dmg-staging

all: $(BUILD_DIR)/$(APP).app $(ZIP) $(DMG)

icons: $(ICONSET)/icon_1024.png

$(ICONSET)/icon_1024.png: $(MASTER) Scripts/MakeIcon.swift
	swift Scripts/MakeIcon.swift --master $(MASTER)

$(BUILD_DIR)/$(APP).app: $(SOURCES) $(ICONSET)/icon_1024.png
	xcodegen generate
	xcodebuild -project $(APP).xcodeproj -scheme $(SCHEME) -configuration $(CONFIG) \
		-derivedDataPath $(DERIVED) build
	rm -rf $@
	cp -R $(BUILT_APP) $@
	codesign --force --deep --sign - $@

$(ZIP): $(BUILD_DIR)/$(APP).app
	cd $(BUILD_DIR) && rm -f $(APP).zip && zip -qry $(APP).zip $(APP).app

$(DMG): $(BUILD_DIR)/$(APP).app
	rm -rf $(STAGE)
	mkdir -p $(STAGE)
	cp -R $(BUILD_DIR)/$(APP).app $(STAGE)/
	ln -s /Applications $(STAGE)/Applications
	hdiutil create -volname $(APP) -srcfolder $(STAGE) -ov -format UDZO $@
	rm -rf $(STAGE)

package: $(ZIP) $(DMG)

test:
	xcodegen generate
	xcodebuild -project $(APP).xcodeproj -scheme $(SCHEME) \
		-derivedDataPath $(DERIVED) test

install: all
	./Scripts/install.sh

clean:
	rm -rf $(BUILD_DIR)
