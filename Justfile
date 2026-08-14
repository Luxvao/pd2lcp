# Builds the appimage
[working-directory: 'crates/pd2lcp-iced']
appimage:
    cargo appimage

# Runs the app
run:
    cargo r

# Builds the container
build_container:
    podman build -t pd2lcp_devenv .

# Builds the files release-ready
build_release: build_container
    cargo clean
    podman run --rm -it -v .:/build pd2lcp_devenv bash -c "cargo b -Zbuild-std --release && cargo xwin b --target x86_64-pc-windows-msvc --release --features=gamemode && cd crates/pd2lcp-iced && cargo appimage -Zbuild-std"
