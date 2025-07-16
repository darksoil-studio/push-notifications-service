{
  description = "Template for Holochain app development";

  inputs = {
    holochain-utils.url = "github:darksoil-studio/holochain-utils/main-0.5";
    nixpkgs.follows = "holochain-utils/nixpkgs";

    service-providers.url = "github:darksoil-studio/service-providers/main-0.5";
    service-providers.inputs.holochain-utils.follows = "holochain-utils";

    clone-manager.url = "github:darksoil-studio/clone-manager-zome/main-0.5";
    clone-manager.inputs.holochain-utils.follows = "holochain-utils";

    garnix-lib = {
      url = "github:garnix-io/garnix-lib";
      inputs.nixpkgs.follows = "nixpkgs";
    };
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
  };

  outputs = inputs:
    inputs.holochain-utils.inputs.holonix.inputs.flake-parts.lib.mkFlake {
      inherit inputs;
    } {

      imports = [
        ./servers.nix
        ./workdir/happ.nix
        ./crates/push_notifications_service_provider/default.nix
        ./crates/push_notifications_service_client/default.nix
        inputs.holochain-utils.outputs.flakeModules.dependencies
        inputs.holochain-utils.outputs.flakeModules.builders
      ];

      systems =
        builtins.attrNames inputs.holochain-utils.inputs.holonix.devShells;
      perSystem = { inputs', config, pkgs, system, self', ... }: {
        devShells.default = pkgs.mkShell {
          inputsFrom = [
            inputs'.holochain-utils.devShells.synchronized-pnpm
            inputs'.holochain-utils.devShells.holochainDev
            inputs'.holochain-utils.devShells.default
          ];

          packages = [
            inputs'.holochain-utils.packages.holochain
            inputs'.holochain-utils.packages.hc-scaffold-zome
            inputs'.holochain-utils.packages.hc-pilot
          ];
        };
        devShells.npm-ci = inputs'.holochain-utils.devShells.synchronized-pnpm;

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
