# To learn more about how to use Nix to configure your environment
# see: https://firebase.google.com/docs/studio/customize-workspace
{ pkgs, ... }: {
  # Which nixpkgs channel to use.
  channel = "unstable"; # or "stable"

  # Use https://search.nixos.org/packages to find packages
  packages = with pkgs; [
    # Rust toolchain
    rustc
    cargo
    rustfmt
    clippy

    # Build dependencies
    pkg-config
    openssl.dev

    # Other useful tools
    gcc
  ];

  # Sets environment variables in the workspace
  env = {};

  idx = {
    # Search for the extensions you want on https://open-vsx.org/ and use "publisher.id"
    extensions = [
      "rust-lang.rust-analyzer"
      "serayuzgur.crates"
      "tamasfe.even-better-toml"
    ];

    # Enable previews
    previews = {
      enable = true;
      previews = {};
    };

    # Workspace lifecycle hooks
    workspace = {
      onCreate = {};
      onStart = {};
    };
  };
}
