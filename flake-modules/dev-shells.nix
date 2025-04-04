{ ... }:
{
  perSystem =
    { pkgs, ... }:
    {
      devShells.default = pkgs.mkShellNoCC {
        packages = with pkgs; [
          nodejs
          (rust-bin.stable.latest.default.override {
            extensions = [
              "rust-src"
              "clippy"
              "rust-analyzer"
            ];
          })
        ];
      };
    };
}
