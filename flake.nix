{
  description = "Emu Pad";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

    flake-utils.url = "github:numtide/flake-utils";

    common.url = "github:liar2357/nix-dev-common";
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
      common,
    }:

    flake-utils.lib.eachDefaultSystem (
      system:

      let
        pkgs = import nixpkgs {
          inherit system;
        };
      in
      {
        #
        # 開発環境
        #
        devShells.default = common.devShells.${system}.rust;

        #
        # パッケージ
        #
        packages.default = pkgs.callPackage ./package.nix { };

        #
        # nix run
        #
        apps.default = {
          type = "app";
          program = "${self.packages.${system}.default}/bin/emu-pad";
        };
      }
    );
}
