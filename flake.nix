{
  description = "Template for Holochain app development";

  inputs = {
    holonix.url = "github:holochain/holonix/main-0.5";
    crane.follows = "holonix/crane";
    nixpkgs.follows = "holonix/nixpkgs";
    flake-parts.follows = "holonix/flake-parts";

    scaffolding.url = "github:darksoil-studio/scaffolding/main-0.5";
    holochain-nix-builders.url =
      "github:darksoil-studio/holochain-nix-builders/main-0.5";
    tauri-plugin-holochain.url =
      "github:darksoil-studio/tauri-plugin-holochain/main-0.5";
    playground.url = "github:darksoil-studio/holochain-playground/main-0.5";

    service-providers.url = "github:darksoil-studio/service-providers/main-0.5";
    clone-manager.url = "github:darksoil-studio/clone-manager-zome/main-0.5";
  };

  nixConfig = {
    extra-substituters = [
      "https://holochain-ci.cachix.org"
      "https://darksoil-studio.cachix.org"
    ];
    extra-trusted-public-keys = [
      "holochain-ci.cachix.org-1:5IUSkZc0aoRS53rfkvH9Kid40NpyjwCMCzwRTXy+QN8="
      "darksoil-studio.cachix.org-1:UEi+aujy44s41XL/pscLw37KEVpTEIn8N/kn7jO8rkc="
    ];
    # To support tests with access to networking
    sandbox = "relaxed";
  };

  outputs = inputs:
    inputs.flake-parts.lib.mkFlake { inherit inputs; } {
      imports = [
        ./workdir/happ.nix
        ./crates/push_notifications_service_provider/default.nix
        ./crates/push_notifications_service_client/default.nix
        inputs.holochain-nix-builders.outputs.flakeModules.builders
      ];

      systems = builtins.attrNames inputs.holonix.devShells;
      perSystem = { inputs', config, pkgs, system, self', ... }: {
        devShells.default = pkgs.mkShell {
          inputsFrom = [
            inputs'.scaffolding.devShells.synchronized-pnpm
            inputs'.holochain-nix-builders.devShells.holochainDev
            inputs'.holonix.devShells.default
          ];

          packages = [
            inputs'.holochain-nix-builders.packages.holochain
            inputs'.scaffolding.packages.hc-scaffold-zome
            inputs'.tauri-plugin-holochain.packages.hc-pilot
            inputs'.playground.packages.hc-playground
          ];
        };
        devShells.npm-ci = inputs'.scaffolding.devShells.synchronized-pnpm;

        packages.test-push-notifications-service = pkgs.writeShellApplication {
          name = "test-push-notifications-service";
          runtimeInputs = [
            self'.packages.push-notifications-service-provider.meta.debug
            self'.packages.push-notifications-service-client.meta.debug
          ];
          text = ''
            export RUST_LOG=''${RUST_LOG:=error}

            DIR1="$(mktemp -d)"
            DIR2="$(mktemp -d)"
            push-notifications-service-provider --bootstrap-url http://bad.bad --data-dir "$DIR1" &
            push-notifications-service-provider --bootstrap-url http://bad.bad --data-dir "$DIR2" &
            push-notifications-service-client --bootstrap-url http://bad.bad publish-service-account-key --service-account-key-path "$1"
            push-notifications-service-client --bootstrap-url http://bad.bad create-clone-request --network-seed "$2"

            echo "The test push notifications service is now ready to be used."

            echo ""

            function cleanup {
              killall push-notifications-service-provider
              rm -rf "$DIR1"
              rm -rf "$DIR2"
            }

            trap cleanup 2 ERR

            wait
            killall push-notifications-service-provider
          '';
        };
      };
    };
}
