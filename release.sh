#!/bin/sh

# Only using the read command to wait for user to press enter
# shellcheck disable=SC2162

set -e

TODAY=$(date --iso-8601)
UNIX_TIME=$(date +%s)
LAST_TAG=$(git describe --tags --abbrev=0 @^)

echo "Last tag was $LAST_TAG, bump ./Cargo.toml version"
read
# Get current version
VERSION=$(sed -n -r 's/version = "(.*)"/\1/p' Cargo.toml | head -n 1)
echo "New release tag will be v$VERSION"
echo "Write a changelog ./static/xyz.ljoonal.neospeeps.metainfo.xml changelog"
CHANGES=$(git log --pretty=format:%s "$LAST_TAG..HEAD")
echo "<release version=\"$VERSION\" date=\"$TODAY\" timestamp=\"$UNIX_TIME\">"
echo "	<url>https://neos.ljoonal.xyz/peeps/releases/tag/v$VERSION</url>"
echo "	<description>"
echo "		<ul>"
echo "$CHANGES" | sed "s/\(.*\)/			<li>\\1<\/li>/g"
echo "		</ul>"
echo "	</description>"
echo "</release>"
echo ""
echo "Modify & add to ./static/xyz.ljoonal.neospeeps.metainfo.xml"
echo "git add . && git commit -m \"Release v$VERSION\""
read
#git tag "$VERSION"

# Build for linux
cargo +stable build --release
# Create sha256 integrity hash
cd target/release && sha256sum neos_peeps > neos_peeps.sha256 && cd ../..

# Build for windows
cargo +stable build --release --target x86_64-pc-windows-gnu

# Windows AV pseudo-requires code to be signed. 
# So doing that with a self-signed cert.
# Also provide a sha256 hash.
rm "target/x86_64-pc-windows-gnu/release/win-neos_peeps.exe" \
	& echo "Signing windows executable" \
	&& osslsigncode sign -h sha256 \
	-in "target/x86_64-pc-windows-gnu/release/neos_peeps.exe" \
	-out "target/x86_64-pc-windows-gnu/release/win-neos_peeps.exe" \
	-pkcs12 "ljoonal.pfx" -askpass \
	&& cd target/x86_64-pc-windows-gnu/release \
	&& sha256sum win-neos_peeps.exe > win-neos_peeps.exe.sha256 \
	&& cd ../../..

CHANGES_MARKDOWN=$(echo "$CHANGES" | sed "s/\(.*\)/- \\1/g")
CHANGES_MARKDOWN="$CHANGES_MARKDOWN\n\n[Full changelog](https://git.ljoonal.xyz/ljoonal/NeosPeeps/compare/$LAST_TAG...v$VERSION)"

echo "$CHANGES_MARKDOWN"

exit 0

# Create a gitea release draft
tea release create --draft --target main \
  --tag "v$VERSION" --title "v$VERSION" \
  -a target/release/neos_peeps \
  -a target/release/neos_peeps.sha256\
  -a target/x86_64-pc-windows-gnu/release/win-neos_peeps.exe \
  -a target/x86_64-pc-windows-gnu/release/win-neos_peeps.exe.sha256


# Upload to itch.io
butler push target/release/neos_peeps ljoonal/neospeeps:linux --userversion "$VERSION"
butler push target/x86_64-pc-windows-gnu/release/win-neos_peeps.exe ljoonal/neospeeps:win --userversion "$VERSION"
