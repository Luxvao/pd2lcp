#! /bin/sh

set -e

cargo b -Zbuild-std --release
cargo xwin b --target x86_64-pc-windows-msvc --release --features=gamemode

cd crates/pd2lcp-iced
cargo appimage -Zbuild-std

cd ../../

mkdir -p releases

cp target/release/pd2lcp-iced releases/
cp target/appimage/pd2lcp-iced.AppImage releases/
cp target/x86_64-pc-windows-msvc/release/pd2lcp-iced.exe releases/

cd releases

sha256sum * > SHA256SUMS
