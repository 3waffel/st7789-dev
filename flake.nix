{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    utils.url = "github:numtide/flake-utils";
    naersk = {
      url = "github:nix-community/naersk/master";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = {
    self,
    nixpkgs,
    utils,
    naersk,
    fenix,
    ...
  }:
    utils.lib.eachDefaultSystem (
      system: let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [
            fenix.overlays.default
            (final: prev: {
              toolchain = with prev.fenix;
                combine [
                  (complete.withComponents [
                    "cargo"
                    "clippy"
                    "rust-src"
                    "rustc"
                    "rustfmt"
                  ])
                ];
            })
          ];
        };
        naersk-lib = with pkgs;
          naersk.lib.${system}.override {
            cargo = toolchain;
            rustc = toolchain;
          };
        pyPkgs = ps:
          with ps; [
            pillow
            spidev
            numpy
            rpi-gpio
          ];
      in rec {
        packages.default = naersk-lib.buildPackage {
          pname = "st7789-dev";
          src = ./.;
        };
        devShells.default = with pkgs;
          mkShell {
            packages = [
              toolchain
              (python3.withPackages pyPkgs)
            ];
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
              after = ["network-online.target"];
              wantedBy = ["multi-user.target"];
              environment = {};
              serviceConfig = {
                DynamicUser = true;
                Restart = "always";
                ExecStart = "${cfg.package}/bin/st7789-dev";
              };
            };
          };
        };
    };
}
