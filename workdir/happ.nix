{ inputs, ... }:

{
  # Import all `../dnas/*/dna.nix` files
  imports = (map (m: "${./..}/dnas/${m}/dna.nix") (builtins.attrNames
    (if builtins.pathExists ../dnas then builtins.readDir ../dnas else { })));

  perSystem = { inputs', lib, self', system, ... }: {
    packages.push_notifications_service_provider_happ =
      inputs.tnesh-stack.outputs.builders.${system}.happ {
        happManifest = ./happ.yaml;

        dnas = {
          push_notifications_service =
            self'.builders.push_notifications_service_dna {
              clone_manager_provider = true;
            };
          service_providers = self'.packages.service_providers_dna;
        };
      };

  };
}
