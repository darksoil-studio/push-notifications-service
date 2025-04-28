{ inputs, ... }:

{
  imports = (map (m: "${./.}/zomes/coordinator/${m}/zome.nix")
    (builtins.attrNames (builtins.readDir ./zomes/coordinator)))
    ++ (map (m: "${./.}/zomes/integrity/${m}/zome.nix")
      (builtins.attrNames (builtins.readDir ./zomes/integrity)));

  perSystem = { inputs', self', lib, system, ... }: {
    builders.push_notifications_service_dna = { clone_manager_provider }:
      inputs.holochain-nix-builders.outputs.builders.${system}.dna {
        dnaManifest = ./workdir/dna.yaml;
        zomes = {
          # This overrides all the "bundled" properties for the DNA manifest
          push_notifications_service_integrity =
            self'.packages.push_notifications_service_integrity;
          push_notifications_service =
            self'.packages.push_notifications_service;
          clone_manager_integrity =
            inputs'.clone-manager.packages.clone_manager_integrity;
          clone_manager = inputs'.clone-manager.builders.clone_manager {
            provider = clone_manager_provider;
          };
        };
      };
  };
}

