{ inputs, self, ... }:

{
  perSystem = { inputs', pkgs, self', lib, system, ... }:
    let
      SERVICE_PROVIDER_HAPP =
        self'.packages.push_notifications_service_provider_happ.meta.debug;
      CLIENT_HAPP =
        self'.packages.push_notifications_service_client_happ.meta.debug;

      END_USER_HAPP = (inputs.holochain-utils.outputs.builders.${system}.happ {
        happManifest = builtins.toFile "happ.yaml" ''
          ---
          manifest_version: "1"
          name: test_happ
          description: ~
          roles:   
            - name: services
              provisioning:
                strategy: create
                deferred: false
              dna:
                bundled: ""
                modifiers:
                  network_seed: ~
                  properties: ~
                version: ~
                clone_limit: 100000
        '';

        dnas = { services = inputs'.service-providers.packages.services_dna; };
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
          inputs.holochain-utils.outputs.dependencies.${system}.holochain.buildInputs;
        LIBCLANG_PATH = "${pkgs.llvmPackages_18.libclang.lib}/lib";
      };
      cargoArtifacts = craneLib.buildDepsOnly commonArgs;
      binary =
        craneLib.buildPackage (commonArgs // { inherit cargoArtifacts; });
      check = craneLib.buildPackage (commonArgs // {
        inherit cargoArtifacts;
        doCheck = true;
        __noChroot = true;
        # RUST_LOG = "info";
        WASM_LOG = "info";
        # For the integration test
        inherit END_USER_HAPP CLIENT_HAPP SERVICE_PROVIDER_HAPP;
      });

      binaryWithDebugHapp =
        pkgs.runCommandLocal "push-notifications-service-provider" {
          buildInputs = [ pkgs.makeWrapper ];
        } ''
          mkdir $out
          mkdir $out/bin
          DNA_HASHES=test
          makeWrapper ${binary}/bin/push-notifications-service-provider $out/bin/push-notifications-service-provider \
            --add-flags "${self'.packages.push_notifications_service_provider_happ.meta.debug} --app-id $DNA_HASHES"
        '';
      binaryWithHapp =
        pkgs.runCommandLocal "push-notifications-service-provider" {
          buildInputs = [ pkgs.makeWrapper ];
          meta = { debug = binaryWithDebugHapp; };
        } ''
          mkdir $out
          mkdir $out/bin
          DNA_HASHES=$(cat ${self'.packages.push_notifications_service_provider_happ.dna_hashes})
          makeWrapper ${binary}/bin/push-notifications-service-provider $out/bin/push-notifications-service-provider \
            --add-flags "${self'.packages.push_notifications_service_provider_happ} --app-id $DNA_HASHES"
        '';
    in rec {

      builders.push-notifications-service-provider = { progenitors }:
        let
          progenitorsArg = builtins.toString
            (builtins.map (p: " --progenitors ${p}") progenitors);

          binaryDebugWithProgenitors =
            pkgs.runCommandLocal "push-notifications-service-provider" {
              buildInputs = [ pkgs.makeWrapper ];
            } ''
              mkdir $out
              mkdir $out/bin
              DNA_HASHES=test
              makeWrapper ${binaryWithDebugHapp}/bin/push-notifications-service-provider $out/bin/push-notifications-service-provider \
                --add-flags "${progenitorsArg}"
            '';
          binaryWithProgenitors =
            pkgs.runCommandLocal "push-notifications-service-provider" {
              buildInputs = [ pkgs.makeWrapper ];
              meta = {
                debug = binaryDebugWithProgenitors;
                inherit cargoArtifacts;
              };
            } ''
              mkdir $out
              mkdir $out/bin
              DNA_HASHES=$(cat ${self'.packages.push_notifications_service_provider_happ.dna_hashes})
              makeWrapper ${binaryWithHapp}/bin/push-notifications-service-provider $out/bin/push-notifications-service-provider \
                --add-flags "${progenitorsArg}"
            '';
        in binaryWithProgenitors;

      packages.push-notifications-service-provider =
        builders.push-notifications-service-provider {
          progenitors = inputs.service-providers.outputs.progenitors;
        };

      checks.send-push-notification-test = check;
    };
}
