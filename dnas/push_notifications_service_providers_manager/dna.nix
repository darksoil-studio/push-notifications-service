{ inputs, ... }:

{
  imports = (map (m: "${./.}/zomes/coordinator/${m}/zome.nix")
    (builtins.attrNames (builtins.readDir ./zomes/coordinator)))
    ++ (map (m: "${./.}/zomes/integrity/${m}/zome.nix")
      (builtins.attrNames (builtins.readDir ./zomes/integrity)));

  perSystem = { inputs', self', lib, system, ... }: {
    packages.push_notifications_service_providers_manager =
      inputs.tnesh-stack.outputs.builders.${system}.dna {
        dnaManifest = ./workdir/dna.yaml;
        zomes = {
          # This overrides all the "bundled" properties for the DNA manifest
          push_notifications_service_providers_manager_integrity =
            self'.packages.push_notifications_service_providers_manager_integrity;
          push_notifications_service_providers_manager =
            self'.packages.push_notifications_service_providers_manager;
        };
      };
  };
}

