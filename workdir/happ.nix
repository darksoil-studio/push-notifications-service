{ inputs, ... }:

{
  # Import all `../dnas/*/dna.nix` files
  imports = (map (m: "${./..}/dnas/${m}/dna.nix") (builtins.attrNames
    (if builtins.pathExists ../dnas then builtins.readDir ../dnas else { })));

  perSystem = { inputs', lib, self', system, ... }: {
    packages.push_notifications_service_provider_happ =
      inputs.holochain-nix-builders.outputs.builders.${system}.happ {
        happManifest = ./happ.yaml;

        dnas = {
          push_notifications_service =
            self'.packages.push_notifications_service_dna;
          services = self'.packages.services_dna;
        };
      };

    packages.push_notifications_service_client_happ =
      inputs.holochain-nix-builders.outputs.builders.${system}.happ {
        happManifest = ./happ.yaml;

        dnas = {
          push_notifications_service =
            self'.packages.push_notifications_service_client_dna;
          services = self'.packages.services_dna;
        };
      };

  };
}
