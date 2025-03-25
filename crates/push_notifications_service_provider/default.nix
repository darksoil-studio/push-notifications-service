{ inputs, self, ... }:

{
  perSystem = { inputs', pkgs, self', lib, system, ... }: {

    packages.push-notifications-service-provider = let
      craneLib = inputs.crane.mkLib pkgs;

      cratePath = ./.;

      cargoToml =
        builtins.fromTOML (builtins.readFile "${cratePath}/Cargo.toml");
      crate = cargoToml.package.name;

      commonArgs = {
        src = craneLib.cleanCargoSource (craneLib.path self.outPath);
        doCheck = false;
        buildInputs =
          inputs.tnesh-stack.outputs.dependencies.${system}.holochain.buildInputs;
      };
      binary = craneLib.buildPackage (commonArgs // {
        pname = crate;
        version = cargoToml.package.version;
      });
      binaryWithHapp =
        pkgs.runCommandLocal "push" { buildInputs = [ pkgs.makeWrapper ]; } ''
          mkdir $out
          mkdir $out/bin
          makeWrapper ${binary}/bin/push-notifications-service-provider $out/bin/push-notifications-service-provider \
            --add-flags "${self'.packages.push_notifications_service_provider_happ}"
        '';
    in binaryWithHapp;

    # builders.aon-for-happs = { happs }:
    #   ;

    # checks.aon-for-happs = let
    #   happ = inputs.tnesh-stack.outputs.builders.${system}.happ {
    #     happManifest = builtins.toFile "happ.yaml" ''
    #       manifest_version: '1'
    #       name: happ-store
    #       description: null
    #       roles: []
    #       allow_deferred_memproofs: false
    #     '';
    #     dnas = { };
    #   };

    # in builders.aon-for-happs { happs = [ happ ]; };
  };
}
