{ inputs, ... }:
{
  imports = [
    inputs.treefmt-nix.flakeModule
  ];
  perSystem.treefmt = {
    projectRootFile = "flake.nix";
    programs.nixfmt.enable = true;
    programs.rustfmt.enable = true;
    settings.global.excludes = [
      ".editorconfig"
      ".envrc"
      ".direnv/*"
      "Cargo.toml"
    ];
  };
}
