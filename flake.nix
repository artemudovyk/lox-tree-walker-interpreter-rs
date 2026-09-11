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
      systems = [
        "x86_64-linux"
      ];

      # Allows to re-configure default nixpkgs
      eachSystem =
        systems: f:
        builtins.foldl' (
          a: s: a // builtins.mapAttrs (k: v: (a.${k} or { }) // { ${s} = v; }) (f s)
        ) { } systems;
    in
    eachSystem systems (
      system:
      let
        overlays = [ (import inputs.rust-overlay) ];
        pkgs = import inputs.nixpkgs {
          inherit system overlays;
          config.allowUnfree = true;
        };
      in
      {
        devShells.default = pkgs.mkShell {
          nativeBuildInputs = [
            pkgs.rust-bin.stable.latest.default
            pkgs.pkg-config
          ];

          depsBuildBuild = [
          ];

          buildInputs = [
            pkgs.openssl
          ];
        };
      }
    );
}
