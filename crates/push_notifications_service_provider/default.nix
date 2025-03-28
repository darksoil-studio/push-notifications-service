{ inputs, self, ... }:

{
  perSystem = { inputs', pkgs, self', lib, system, ... }:
    let
      SERVICE_PROVIDER_HAPP =
        self'.packages.push_notifications_service_provider_happ.meta.debug;
      END_USER_HAPP = (inputs.tnesh-stack.outputs.builders.${system}.happ {
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
      }).meta.debug;
      INFRA_PROVIDER_HAPP =
        (inputs.tnesh-stack.outputs.builders.${system}.happ {
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
        }).meta.debug;

      HAPP_DEVELOPER_HAPP =
        (inputs.tnesh-stack.outputs.builders.${system}.happ {
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
        }).meta.debug;

      craneLib = inputs.crane.mkLib pkgs;
      src = craneLib.cleanCargoSource (craneLib.path self.outPath);

      cratePath = ./.;

      cargoToml =
        builtins.fromTOML (builtins.readFile "${cratePath}/Cargo.toml");
      crate = cargoToml.package.name;
      pname = crate;
      version = cargoToml.package.version;

      commonArgs = {
        inherit src version pname;
        doCheck = false;
        buildInputs =
          inputs.tnesh-stack.outputs.dependencies.${system}.holochain.buildInputs;
      };
      cargoArtifacts = craneLib.buildDepsOnly commonArgs;
      binary =
        craneLib.buildPackage (commonArgs // { inherit cargoArtifacts; });
      check = craneLib.buildPackage (commonArgs // {
        inherit cargoArtifacts;
        doCheck = true;
        # sandbox = false;
        __noChroot = true;
        # RUST_LOG = "info";
        WASM_LOG = "info";
        # cargoTestExtraArgs = "--no-run -- --nocapture";
        # For the integration test
        inherit END_USER_HAPP INFRA_PROVIDER_HAPP SERVICE_PROVIDER_HAPP
          HAPP_DEVELOPER_HAPP;
      });
    in {

      packages.push-notifications-service-provider = let
        binaryWithHapp =
          pkgs.runCommandLocal "push-notifications-service-provider" {
            buildInputs = [ pkgs.makeWrapper ];
          } ''
            mkdir $out
            mkdir $out/bin
            makeWrapper ${binary}/bin/push-notifications-service-provider $out/bin/push-notifications-service-provider \
              --add-flags "${self'.packages.push_notifications_service_provider_happ}"
          '';
      in binaryWithHapp;

      checks.send-push-notification-test = check;
    };
}
