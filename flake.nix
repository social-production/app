{
  description = "Social Production — P2P app (Holochain 0.7)";

  # Prebuilt holochain/hc/lair from Holochain's cache. Needs the cache listed in
  # trusted-substituters (see README "Getting started"), then `--accept-flake-config`.
  nixConfig = {
    extra-substituters = [ "https://holochain-ci.cachix.org" ];
    extra-trusted-public-keys = [ "holochain-ci.cachix.org-1:5IUSkZc0aoRS53rfkvH9Kid40NpyjwCMCzwRTXy+QN8=" ];
  };

  inputs = {
    # Pinned to the 0.7 release series; `nix flake update holonix` moves within it.
    holonix.url = "github:holochain/holonix?ref=main-0.7";

    nixpkgs.follows = "holonix/nixpkgs";
    flake-parts.follows = "holonix/flake-parts";
  };

  outputs = inputs@{ flake-parts, ... }: flake-parts.lib.mkFlake { inherit inputs; } {
    systems = builtins.attrNames inputs.holonix.devShells;
    perSystem = { inputs', pkgs, lib, ... }: {
      formatter = pkgs.nixpkgs-fmt;

      devShells.default = pkgs.mkShell {
        packages = (with inputs'.holonix.packages; [
          holochain
          hc
          bootstrap-srv
          lair-keystore
          rust # includes the wasm32-unknown-unknown target for zomes
        ]) ++ (with pkgs; [
          nodejs_24
          binaryen
          pkg-config
          openssl
          openssl.dev
        ]) ++ lib.optionals pkgs.stdenv.isLinux (with pkgs; [
          # Tauri desktop build dependencies (native/)
          webkitgtk_4_1
          gtk3
          libsoup_3
          librsvg
        ]);

        shellHook = ''
          # stdenv adds `-rpath $out/lib` with $out under the project dir; the workspace
          # path ("Social Production") has a space, which splits that flag and breaks
          # every link. A dev shell never installs to $out, so drop it.
          export NIX_LDFLAGS="''${NIX_LDFLAGS/-rpath $out\/lib/}"
          # Link the Nix-provided OpenSSL instead of vendoring (vendored build also
          # trips over the space in the path).
          export OPENSSL_NO_VENDOR=1
          export PS1='\[\033[1;32m\][sp:\w]\$\[\033[0m\] '
        '';
      };
    };
  };
}
