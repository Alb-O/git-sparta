{ inputs, ... }:
{
  imports = [
    inputs.rust-flake.flakeModules.default
    inputs.rust-flake.flakeModules.nixpkgs
  ];
  perSystem = { config, self', pkgs, lib, ... }: {
    rust-project.crates."git-sparta".crane.args = {
      buildInputs = lib.optionals pkgs.stdenv.isDarwin (
        with pkgs.darwin.apple_sdk.frameworks; [
          IOKit
        ]
      );
    };
    # Ensure the path for the top-level crate is set so default-crates logic
    # doesn't try to read an undefined path (this fixes errors when the crate
    # is referenced by the rust-flake module).
    rust-project.crates."git-sparta".path = ../../.;
    packages.default = self'.packages.git-sparta;
  };
}
