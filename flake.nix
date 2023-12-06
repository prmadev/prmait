{
  description = "Building static binaries with musl";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    flake-utils.url = "github:numtide/flake-utils";

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
        flake-utils.follows = "flake-utils";
      };
    };

    advisory-db = {
      url = "github:rustsec/advisory-db";
      flake = false;
    };
  };

  outputs = {
    nixpkgs,
    crane,
    flake-utils,
    rust-overlay,
    advisory-db,
    ...
  }:
    flake-utils.lib.eachSystem ["x86_64-linux"] (system: let
      pkgs = import nixpkgs {
        inherit system;
        overlays = [(import rust-overlay)];
      };
      inherit (pkgs) lib;
      src = craneLib.cleanCargoSource (craneLib.path ./.);
      commonArgs = {
        inherit src;

        buildInputs =
          [
            # Add additional build inputs here
          ]
          ++ lib.optionals pkgs.stdenv.isDarwin [
            # Additional darwin specific inputs can be set here
            pkgs.libiconv
          ];

        # Additional environment variables can be set directly
      };
      rustToolchain = pkgs.rust-bin.stable.latest.default.override {
        targets = ["x86_64-unknown-linux-musl"];
      };

      craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;

      cargoArtifacts = craneLib.buildDepsOnly commonArgs;
      rvr = craneLib.buildPackage {
        inherit src;
        CARGO_BUILD_TARGET = "x86_64-unknown-linux-musl";
        CARGO_BUILD_RUSTFLAGS = "-C target-feature=+crt-static";
        name = "rvr";
        pname = "rvr";
        version = "0.1";
        cargoExtraArgs = "--bin rvr --locked";
      };
      tsk = craneLib.buildPackage {
        inherit src;
        CARGO_BUILD_TARGET = "x86_64-unknown-linux-musl";
        CARGO_BUILD_RUSTFLAGS = "-C target-feature=+crt-static";
        name = "tsk";
        pname = "tsk";
        version = "0.1";
        cargoExtraArgs = "--bin tsk --locked";
      };
      jnl = craneLib.buildPackage {
        inherit src;
        CARGO_BUILD_TARGET = "x86_64-unknown-linux-musl";
        CARGO_BUILD_RUSTFLAGS = "-C target-feature=+crt-static";
        name = "jnl";
        pname = "jnl";
        version = "0.1";
        cargoExtraArgs = "--bin jnl  --locked";
      };
    in {
      inherit jnl;
      inherit tsk;
      inherit rvr;
      checks = {
        my-crate-clippy = craneLib.cargoClippy (commonArgs
          // {
            inherit cargoArtifacts;
            cargoClippyExtraArgs = "-- --deny warnings";
          });

        my-crate-doc = craneLib.cargoDoc (commonArgs
          // {
            inherit cargoArtifacts;
          });

        # Check formatting
        my-crate-fmt = craneLib.cargoFmt {
          inherit src;
        };

        # Audit dependencies
        my-crate-audit = craneLib.cargoAudit {
          inherit src advisory-db;
        };

        # Audit licenses
        my-crate-deny = craneLib.cargoDeny {
          inherit src;
        };

        # Run tests with cargo-nextest
        # Consider setting `doCheck = false` on `my-crate` if you do not want
        # the tests to run twice
        my-crate-nextest = craneLib.cargoNextest (commonArgs
          // {
            inherit cargoArtifacts;
            partitions = 1;
            partitionType = "count";
          });
      };

      packages.tsk = tsk;
      packages.jnl = jnl;
      packages.rvr = rvr;
      apps.jnl = flake-utils.lib.mkApp {
        name = "jnl";
        drv = jnl;
      };
      apps.rvr = flake-utils.lib.mkApp {
        name = "rvr";
        drv = rvr;
      };
      apps.tsk = flake-utils.lib.mkApp {
        name = "rvr";
        drv = tsk;
      };
    });
}
