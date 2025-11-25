{
  lib,
  ...
}:
{
  perSystem =
    { pkgs, rustPlatform, ... }:
    let
      src = pkgs.nix-gitignore.gitignoreSource [
        ".git"
        ".github"
        "target"
        "result"
        "result-*"
        "tmp"
      ] ../../.;

      cargoLock = {
        lockFile = ../../Cargo.lock;
      };

      gitSpartaPkg = rustPlatform.buildRustPackage {
        pname = "git-sparta";
        version = "0.1.0";
        inherit src cargoLock;
        nativeCheckInputs = [ pkgs.git ];
        buildInputs = lib.optionals pkgs.stdenv.isDarwin (with pkgs.darwin.apple_sdk.frameworks; [ IOKit ]);
        cargoBuildFlags = [ "--locked" ];
        postInstall = ''
          for link in sparta sparta-tags sparta-setup sparta-teardown; do
            ln -s $out/bin/git-sparta $out/bin/$link
          done
        '';
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
