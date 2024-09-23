{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    utils.url = "github:numtide/flake-utils";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    crane.url = "github:ipetkov/crane";
  };

  outputs = {
    self,
    nixpkgs,
    utils,
    fenix,
    crane,
    ...
  }:
    utils.lib.eachDefaultSystem (
      system: let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [
            fenix.overlays.default
            (final: prev: {
              fenixToolchain = with prev.fenix;
                combine [
                  stable.clippy
                  stable.rustc
                  stable.cargo
                  stable.rustfmt
                  stable.rust-src
                ];
            })
          ];
        };
        _crane = (crane.mkLib pkgs).overrideToolchain pkgs.fenixToolchain;
        pyPkgs = ps:
          with ps; [
            pillow
            spidev
            numpy
            rpi-gpio
          ];
        env = {
          DEP_LV_CONFIG_PATH = "${self}/include";
          LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";
        };
      in rec {
        packages.default = packages.fromCrane;
        packages.fromCrane = with pkgs;
          _crane.buildPackage {
            pname = "st7789-dev";
            src = lib.cleanSource ./.;
            doCheck = false;
            buildInputs = [
              clang
              gcc.cc.lib
            ];
            nativeBuildInputs = [
              pkg-config
              rustPlatform.bindgenHook
            ];
            inherit env;
          };
        devShells.default = with pkgs;
          mkShell {
            packages = [
              fenixToolchain
              cargo-watch
              rust-analyzer

              pkg-config
              clang
              gcc.cc.lib
              (python3.withPackages pyPkgs)
            ];
            nativeBuildInputs = [
              rustPlatform.bindgenHook
            ];
            inherit env;
          };
      }
    )
    // rec {
      nixosModules.default = {
        config,
        lib,
        pkgs,
        ...
      }: let
        cfg = config.services.st7789-dev;
        out = self.packages.${pkgs.system}.default;
      in
        with lib; {
          options.services.st7789-dev = {
            enable = mkEnableOption "st7789-dev service";
            package = mkOption {
              type = types.package;
              default = out;
            };
          };

          config = mkIf cfg.enable {
            systemd.services.st7789-dev = {
              wantedBy = ["multi-user.target"];
              environment = {};
              serviceConfig = {
                User = "root";
                Restart = "always";
                ExecStart = "${cfg.package}/bin/st7789-dev";
              };
            };
          };
        };
    };
}
