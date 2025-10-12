{ inputs, ... }:
{
  perSystem = { pkgs, ... }: {
    devShells.default =
      let
        toolchain = inputs.fenix.packages.${pkgs.system}.toolchainOf {
          channel = "nightly";
          date = "2025-10-01";
          sha256 = "sha256-GCGEXGZeJySLND0KU5TdtTrqFV76TF3UdvAHSUegSsk=";
        };
      in
      pkgs.mkShell {
        name = "git-sparta-shell";
        packages = [
          toolchain.cargo
          toolchain.rustc
          toolchain.rustfmt
          toolchain.clippy
          pkgs.just
          pkgs.nixd
        ];
        RUSTC = "${toolchain.rustc}/bin/rustc";
        CARGO = "${toolchain.cargo}/bin/cargo";
      };
  };
}
