{
  description = "Rust environment";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { nixpkgs, rust-overlay, ... }:
  let
    system = "x86_64-linux";

    pkgs = import nixpkgs {
      inherit system;
      overlays = [ rust-overlay.overlays.default ];
    };

    gtkPcPkgs = with pkgs; [
      glib
      gtk4
      cairo
      pango
      gdk-pixbuf
      graphene
      harfbuzz
      libadwaita
      atk
      fribidi
      fontconfig
      freetype
      vulkan-loader
      wayland
      xkeyboard-config
      libxkbcommon
    ];

    fhs = pkgs.buildFHSEnv {
      name = "fhs-devshell";

      targetPkgs = pkgs: [
        (pkgs.rust-bin.stable.latest.default.override {
          extensions = [
            "rust-src"
            "rustfmt"
            "clippy"
            "rust-analyzer"
          ];
        })

        # C stuff
        pkgs.pkg-config
        pkgs.gcc
        pkgs.binutils

        # 32-bit runtime support (for 32-bit Wine)
        pkgs.pkgsi686Linux.glibc
      ] ++ gtkPcPkgs;

      profile = ''
        export PKG_CONFIG_PATH=${pkgs.lib.makeSearchPathOutput "dev" "lib/pkgconfig" gtkPcPkgs}:$PKG_CONFIG_PATH
      '';

      runScript = "zsh";
    };

  in {
    devShells.${system}.default = fhs.env;
  };
}
