{
  inputs = {
    naersk.url = "github:nix-community/naersk/master";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    utils.url = "github:numtide/flake-utils";
    nixvim = {
      url = "github:nix-community/nixvim";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    nvim = {
      url = "github:IcyTv/nvim.nix";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.nixvim.follows = "nixvim";
    };
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = {
    self,
    nixpkgs,
    utils,
    naersk,
    nvim,
    fenix,
    ...
  }:
    utils.lib.eachDefaultSystem (
      system: let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [fenix.overlays.default];
        };
        naersk-lib = pkgs.callPackage naersk {};
        customNvim = nvim.lib.${system}.makeNeovimWithLanguages {
          inherit pkgs;
          languages.rust.enable = true;
          languages.nix.enable = true;
        };
        lucideIcons = pkgs.fetchzip {
          url = "https://github.com/lucide-icons/lucide/releases/download/0.561.0/lucide-icons-0.561.0.zip";
          sha256 = "sha256-ReN9IKZMBuSlkKTsG6JEYPQi5ctirXv54t+Q5h5PaX4=";
        };
        simpleIcons = pkgs.fetchFromGitHub {
          owner = "simple-icons";
          repo = "simple-icons";
          rev = "16.2.0";
          hash = "sha256-bDOiWqonxrcuc5fLvm6p+Y0KpcKlrZibaLROkpfA+PU=";
        };
      in {
        defaultPackage = naersk-lib.buildPackage ./.;
        devShell = with pkgs;
          mkShell {
            buildInputs = [astal.io astal.tray astal.mpris astal.cava astal.wireplumber astal.network astal.bluetooth gtk4 astal.astal4 gtk4-layer-shell json-glib networkmanager adwaita-icon-theme graphene];
            nativeBuildInputs = [
              (pkgs.fenix.complete.withComponents [
                "cargo"
                "clippy"
                "rustc"
                "rust-src"
                "rustfmt"
              ])
              customNvim
              pkg-config
              pre-commit
              ags
              blueprint-compiler
            ];
            RUST_SRC_PATH = rustPlatform.rustLibSrc;

            LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath [
              astal.io
              astal.tray
              astal.mpris
              astal.cava
              astal.wireplumber
              astal.network
              astal.bluetooth
              astal.astal4
              gtk4
              gtk4-layer-shell
              glib
              json-glib
              networkmanager
              graphene
            ];

            LUCIDE_ICONS_PATH = "${lucideIcons}";
            SIMPLE_ICONS_PATH = "${simpleIcons}";
          };
      }
    );
}
