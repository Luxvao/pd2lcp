FROM ubuntu:22.04

RUN apt-get update
RUN apt-get upgrade -y

RUN apt-get install build-essential pkg-config libwayland-dev libx11-dev libxcursor-dev libxrandr-dev libvulkan-dev libxkbcommon-dev curl file appstream llvm clang -y

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- --default-toolchain nightly -y

ENV PATH="/root/.cargo/bin:$PATH"

RUN cargo install cargo-appimage
RUN cargo install cargo-xwin

RUN rustup component add rust-src
RUN rustup target add x86_64-pc-windows-msvc

RUN curl -L -o /appimagetool "https://github.com/AppImage/appimagetool/releases/download/continuous/appimagetool-x86_64.AppImage"

RUN chmod +x /appimagetool

COPY ./appimagetool-wrapper.sh /usr/local/bin/appimagetool

RUN chmod +x /usr/local/bin/appimagetool

WORKDIR /build

