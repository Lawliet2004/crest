#!/usr/bin/env bash
set -euo pipefail

cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.."
project=$PWD
if [[ $(uname -m) != x86_64 ]] || [[ $(uname -s) != Linux ]]; then
    printf '%s\n' 'This package targets Linux x86-64.' >&2
    exit 1
fi

cargo build --locked --release --target-dir target -j 2 -p crest-browser
runtime=$project/target/release
version=$("$runtime/crest-browser" --version)
version=${version##* }
output=$project/dist/crest-browser_${version}_amd64.deb
mkdir -p -- "$project/dist"

stage=$(mktemp -d "$project/dist/.crest-package.XXXXXX")
trap 'rm -rf -- "$stage"' EXIT
package=$stage/package
app=$package/usr/lib/crest-browser
doc=$package/usr/share/doc/crest-browser
mkdir -p -- "$app" "$doc" "$package/DEBIAN" "$package/usr/bin" "$stage/debian"

for file in crest-browser libcef.so chrome_100_percent.pak chrome_200_percent.pak \
    resources.pak icudtl.dat v8_context_snapshot.bin libvulkan.so.1 \
    libvk_swiftshader.so vk_swiftshader_icd.json locales; do
    cp -a --reflink=auto -- "$runtime/$file" "$app/"
done
# The local package uses Chromium's user-namespace sandbox.
ln -s ../lib/crest-browser/crest-browser "$package/usr/bin/crest-browser"
install -Dm644 packaging/crest-browser.desktop "$package/usr/share/applications/crest-browser.desktop"
install -Dm644 packaging/crest-browser.svg "$package/usr/share/icons/hicolor/scalable/apps/crest-browser.svg"
install -m644 packaging/CEF-LICENSE.txt packaging/cef-rs-LICENSE-MIT.txt "$runtime/CREDITS.html" "$doc/"
install -m644 BUILD.md "$doc/BUILD.md"
cargo metadata --locked --format-version 1 --filter-platform x86_64-unknown-linux-gnu \
    | python3 packaging/rust-notices.py > "$doc/rust-notices.txt"

cat > "$stage/debian/control" <<'CONTROL'
Source: crest-browser

Package: crest-browser
Architecture: amd64
Description: Crest Browser
CONTROL
shlibs=$(
    cd -- "$stage"
    dpkg-shlibdeps -O --ignore-missing-info -l"$app" -e"$app/crest-browser" -e"$app/libcef.so"
)
depends=${shlibs#shlibs:Depends=}
installed_size=$(du -sk -- "$package" | cut -f1)
cat > "$package/DEBIAN/control" <<CONTROL
Package: crest-browser
Version: $version
Architecture: amd64
Maintainer: Crest Browser local build <crest-browser@localhost>
Section: web
Priority: optional
Installed-Size: $installed_size
Depends: $depends, libgtk-3-0t64 | libgtk-3-0, ca-certificates
Description: Rust-first desktop web browser powered by Chromium
 Linux development milestone of Crest Browser with native controls,
 tabs, bookmarks, history, downloads and integrated DevTools.
CONTROL
printf '%s\n' 'CEF, Chromium, and Rust dependency notices are in this directory.' > "$doc/copyright"
dpkg-deb --root-owner-group -Zzstd -z3 --build "$package" "$stage/crest-browser.deb"
mv -- "$stage/crest-browser.deb" "$output"
printf '%s\n' "$output"
