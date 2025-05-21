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
      "assets/*"
      "migrations/*"
      "*.ftl"
    ];
    programs.nixfmt.enable = true;
    programs.rustfmt.enable = true;
    programs.prettier.enable = true;
    programs.sqruff.enable = true;
    programs.taplo.enable = true;
    programs.yamlfmt.enable = true;
    programs.hclfmt.enable = true;
    settings.formatter.taplo.options = [
      "--option"
      "array_auto_expand=false"
    ];
  };
}
