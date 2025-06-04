{ inputs, self, ... }:

{
  perSystem = { inputs', pkgs, self', lib, system, ... }:
    let

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
          inputs.holochain-nix-builders.outputs.dependencies.${system}.holochain.buildInputs;
        LIBCLANG_PATH = "${pkgs.llvmPackages_18.libclang.lib}/lib";
      };
      cargoArtifacts = craneLib.buildDepsOnly commonArgs;
      binary =
        craneLib.buildPackage (commonArgs // { inherit cargoArtifacts; });
    in rec {

      builders.push-notifications-service-client = { progenitors }:
        let
          progenitorsArg = builtins.toString
            (builtins.map (p: " --progenitors ${p}") progenitors);

          binaryWithDebugHapp =
            pkgs.runCommandLocal "push-notifications-service-client" {
              buildInputs = [ pkgs.makeWrapper ];
            } ''
              mkdir $out
              mkdir $out/bin
              makeWrapper ${binary}/bin/push-notifications-service-client $out/bin/push-notifications-service-client \
                --add-flags "${self'.packages.push_notifications_service_client_happ.meta.debug} ${progenitorsArg}"
            '';
          binaryWithHapp =
            pkgs.runCommandLocal "push-notifications-service-client" {
              buildInputs = [ pkgs.makeWrapper ];
              meta.debug = binaryWithDebugHapp;
            } ''
              mkdir $out
              mkdir $out/bin
              makeWrapper ${binary}/bin/push-notifications-service-client $out/bin/push-notifications-service-client \
                --add-flags "${self'.packages.push_notifications_service_client_happ} ${progenitorsArg}"
            '';
        in binaryWithHapp;

      packages.test-push-notifications-service-client =
        builders.push-notifications-service-client {
          progenitors =
            [ "uhCAk13OZ84d5HFum5PZYcl61kHHMfL2EJ4yNbHwSp4vn6QeOdFii" ];
        };

      packages.darksoil-push-notifications-service-client =
        builders.push-notifications-service-client {
          progenitors =
            [ "uhCAk13OZ84d5HFum5PZYcl61kHHMfL2EJ4yNbHwSp4vn6QeOdFii" ];
        };
    };
}
