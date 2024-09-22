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
        _naersk = with pkgs;
          naersk.lib.${system}.override {
            cargo = fenixToolchain;
            rustc = fenixToolchain;
          };
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
        packages.default = packages.fromNaive;
        packages.fromNaersk = with pkgs;
          _naersk.buildPackage {
            pname = "st7789-dev";
            src = ./.;
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
        packages.fromNaive = with pkgs;
          rustPlatform.buildRustPackage {
            pname = "st7789-dev";
            inherit ((lib.importTOML ./Cargo.toml).package) version;
            doCheck = false;
            src = lib.cleanSource ./.;
            cargoLock = {
              lockFile = ./Cargo.lock;
              allowBuiltinFetchGit = true;
            };
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
