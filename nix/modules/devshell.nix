{ ... }:
{
  perSystem =
    { pkgs, toolchain, ... }:
    {
      devShells.default = pkgs.mkShell {
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
