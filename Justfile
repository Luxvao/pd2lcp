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
    rm -fr releases/
    podman run --rm -it -v .:/build pd2lcp_devenv
