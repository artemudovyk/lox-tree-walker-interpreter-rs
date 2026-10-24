{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    inputs:
    let
      # Uses Nixpkgs' internal exposed system list instead of hardcoded strings
      eachSystem =
        f:
        inputs.nixpkgs.lib.genAttrs inputs.nixpkgs.lib.systems.flakeExposed (
          system:
          f (
            import inputs.nixpkgs {
              inherit system;
              overlays = [ (import inputs.rust-overlay) ];
              config.allowUnfree = true;
            }
          )
        );
    in
    {
      devShells = eachSystem (pkgs: {
        default = pkgs.mkShell {
          nativeBuildInputs = with pkgs; [
            rust-bin.stable.latest.default
            pkg-config
            cargo-nextest
            vscode-extensions.vadimcn.vscode-lldb.adapter
          ];
          depsBuildBuild = [ ];
          buildInputs = [ ];
        };
      });
    };
}
