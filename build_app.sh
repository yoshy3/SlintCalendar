#!/bin/bash

# Exit on error
set -e

if [ -z "$1" ]; then
    echo "Usage: $0 <target>"
    exit 1
fi

TARGET=$1

echo "Creating bundle for target $TARGET..."

# The name of the app and executable
APP_NAME="SlintCalendar.app"
EXECUTABLE_NAME="slint-calendar"
CARGO_TARGET_DIR="target/$TARGET/release"
BUNDLE_DIR="target/release/$APP_NAME"

# Clean up previous bundle
rm -rf "$BUNDLE_DIR"

echo "Creating bundle directory structure..."
mkdir -p "$BUNDLE_DIR/Contents/MacOS"
mkdir -p "$BUNDLE_DIR/Contents/Resources"

echo "Copying executable..."
cp "$CARGO_TARGET_DIR/$EXECUTABLE_NAME" "$BUNDLE_DIR/Contents/MacOS/$EXECUTABLE_NAME"

echo "Creating Info.plist..."
# Get version from Cargo.toml
VERSION=$(grep '^version' Cargo.toml | head -n 1 | sed 's/version = "\(.*\)"/\1/')

cat > "$BUNDLE_DIR/Contents/Info.plist" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleExecutable</key>
    <string>$EXECUTABLE_NAME</string>
    <key>CFBundleIdentifier</key>
    <string>com.slint.slint-calendar</string>
    <key>CFBundleName</key>
    <string>SlintCalendar</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleVersion</key>
    <string>$VERSION</string>
    <key>CFBundleShortVersionString</key>
    <string>$VERSION</string>
    <key>LSMinimumSystemVersion</key>
    <string>10.12</string>
    <key>CFBundleIconFile</key>
    <string>icon.icns</string>
</dict>
</plist>
EOF

echo "Creating icon..."
ICONSET_DIR="temp/icon.iconset"
mkdir -p "$ICONSET_DIR"
sips --resampleWidth 16 icon.png --out "$ICONSET_DIR/icon_16x16.png" > /dev/null 2>&1
sips --resampleWidth 32 icon.png --out "$ICONSET_DIR/icon_16x16@2x.png" > /dev/null 2>&1
sips --resampleWidth 32 icon.png --out "$ICONSET_DIR/icon_32x32.png" > /dev/null 2>&1
sips --resampleWidth 64 icon.png --out "$ICONSET_DIR/icon_32x32@2x.png" > /dev/null 2>&1
sips --resampleWidth 128 icon.png --out "$ICONSET_DIR/icon_128x128.png" > /dev/null 2>&1
sips --resampleWidth 256 icon.png --out "$ICONSET_DIR/icon_128x128@2x.png" > /dev/null 2>&1
sips --resampleWidth 256 icon.png --out "$ICONSET_DIR/icon_256x256.png" > /dev/null 2>&1
sips --resampleWidth 512 icon.png --out "$ICONSET_DIR/icon_256x256@2x.png" > /dev/null 2>&1
iconutil -c icns "$ICONSET_DIR" -o "$BUNDLE_DIR/Contents/Resources/icon.icns"
rm -r "$ICONSET_DIR"

echo "Successfully created $APP_NAME in target/release/"
