{ inputs, ... }:

{
  perSystem =
    { inputs'
    , self'
    , lib
    , system
    , ...
    }: {
      packages.push_notifications_service_provider_test_dna = inputs.tnesh-stack.outputs.builders.${system}.dna {
        dnaManifest = ./dna.yaml;
        zomes = {
          # Include here the zome packages for this DNA, e.g.:
          profiles_integrity = inputs'.profiles-zome.packages.profiles_integrity;
          profiles = inputs'.profiles-zome.packages.profiles;
          # This overrides all the "bundled" properties for the DNA manifest
          push_notifications_service_provider_integrity = self'.packages.push_notifications_service_provider_integrity;
          push_notifications_service_provider = self'.packages.push_notifications_service_provider;
        };
      };
    };
}

