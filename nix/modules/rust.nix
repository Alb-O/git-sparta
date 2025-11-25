{ inputs, ... }:
{
  perSystem =
    { pkgs, ... }:
    let
      toolchain = inputs.fenix.packages.${pkgs.system}.toolchainOf {
        channel = "nightly";
        date = "2025-10-01";
        sha256 = "sha256-GCGEXGZeJySLND0KU5TdtTrqFV76TF3UdvAHSUegSsk=";
      };

      rustPlatform = pkgs.makeRustPlatform {
        cargo = toolchain.cargo;
        rustc = toolchain.rustc;
      };
    in
    {
      _module.args = {
        inherit toolchain rustPlatform;
      };
    };
}
