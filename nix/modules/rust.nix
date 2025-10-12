{ inputs, lib, ... }:
{
  perSystem =
    { pkgs, ... }:
    let
      baseSrc = builtins.path {
        path = ../../.;
        name = "git-sparta-src";
      };

      rizSrc = builtins.path {
        path = inputs.riz;
        name = "riz-src";
      };

      src = pkgs.runCommand "git-sparta-src" { } ''
        tmp=$TMPDIR/src
        mkdir -p "$tmp"
        cp -R ${baseSrc}/. "$tmp"
        chmod -R u+w "$tmp"
        rm -rf "$tmp/.git" "$tmp/.github" "$tmp/result"
        rm -rf "$tmp"/result-*
        rm -rf "$tmp/target" "$tmp/tmp"
        rm -rf "$tmp/crates/riz"
        mkdir -p "$tmp/crates"
        cp -R ${rizSrc}/. "$tmp/crates/riz"
        rm -rf "$tmp/crates/riz/.git"
        mkdir -p "$out"
        cp -R "$tmp"/. "$out"
      '';

      cargoLock = {
        lockFile = ../../Cargo.lock;
        outputHashes = {
          "frizbee-0.5.0" = "sha256-1zg+rOCysXTKpvxKl80Eer3dijFeo2PWqtUqTRH5puA=";
        };
      };

      gitSpartaPkg =
        pkgs.rustPlatform.buildRustPackage {
          pname = "git-sparta";
          version = "0.1.0";
          inherit src cargoLock;
          buildInputs = lib.optionals pkgs.stdenv.isDarwin (
            with pkgs.darwin.apple_sdk.frameworks;
            [ IOKit ]
          );
          cargoBuildFlags = [ "--locked" ];
          RUSTC_BOOTSTRAP = 1;
        };
    in
    {
      packages.git-sparta = gitSpartaPkg;
      packages.default = gitSpartaPkg;

      apps.git-sparta = {
        type = "app";
        program = lib.getExe gitSpartaPkg;
      };
    };
}
