{ inputs, ... }:

{
  # Import all `../dnas/*/dna.nix` files
  imports = (map (m: "${./..}/dnas/${m}/dna.nix") (builtins.attrNames
    (if builtins.pathExists ../dnas then builtins.readDir ../dnas else { })));

  perSystem = { inputs', lib, self', system, ... }: {
    packages.end_user_test_happ =
      inputs.tnesh-stack.outputs.builders.${system}.happ {
        happManifest = builtins.toFile "happ.yaml" ''
          ---
          manifest_version: "1"
          name: test_happ
          description: ~
          roles:   
            - name: service_providers
              provisioning:
                strategy: create
                deferred: false
              dna:
                bundled: ""
                modifiers:
                  network_seed: ~
                  properties: ~
                  origin_time: ~
                version: ~
                clone_limit: 100000
        '';

        dnas = { service_providers = self'.packages.service_providers_dna; };
      };
    packages.infra_provider_test_happ =
      inputs.tnesh-stack.outputs.builders.${system}.happ {
        happManifest = builtins.toFile "happ.yaml" ''
          ---
          manifest_version: "1"
          name: infra_provider_test_happ
          description: ~
          roles:   
            - name: push_notifications_service_providers_manager
              provisioning:
                strategy: create
                deferred: false
              dna:
                bundled: ""
                modifiers:
                  network_seed: ~
                  properties: ~
                  origin_time: ~
                version: ~
                clone_limit: 0
        '';

        dnas = {
          push_notifications_service_providers_manager =
            self'.packages.push_notifications_service_providers_manager_dna;
        };
      };
    packages.happ_developer_test_happ =
      inputs.tnesh-stack.outputs.builders.${system}.happ {
        happManifest = builtins.toFile "happ.yaml" ''
          ---
          manifest_version: "1"
          name: happ_developer_test_happ
          description: ~
          roles:   
            - name: service_providers
              provisioning:
                strategy: create
                deferred: false
              dna:
                bundled: ""
                modifiers:
                  network_seed: ~
                  properties: ~
                  origin_time: ~
                version: ~
                clone_limit: 100000
        '';

        dnas = { service_providers = self'.packages.service_providers_dna; };
      };

  };
}
