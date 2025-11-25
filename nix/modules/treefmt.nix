{ inputs, ... }:
{
  imports = [ inputs.treefmt-nix.flakeModule ];

  perSystem =
    { ... }:
    {
      treefmt = {
        projectRootFile = "flake.nix";
        programs.rustfmt.enable = true;
        programs.nixfmt.enable = true;
      };
    };
}
