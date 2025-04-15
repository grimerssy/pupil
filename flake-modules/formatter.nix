{ inputs, ... }:
{
  imports = [
    inputs.treefmt-nix.flakeModule
  ];
  perSystem.treefmt = {
    projectRootFile = "flake.nix";
    settings.global.excludes = [
      ".editorconfig"
      ".envrc"
      ".direnv/*"
    ];
    programs.nixfmt.enable = true;
    programs.rustfmt.enable = true;
    programs.prettier.enable = true;
    programs.taplo.enable = true;
    programs.yamlfmt.enable = true;
    settings.formatter.taplo.options = [
      "--option"
      "array_auto_expand=false"
    ];
  };
}
