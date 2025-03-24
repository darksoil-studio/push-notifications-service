{ inputs, ... }:

{
  perSystem =
    { inputs'
    , lib
    , self'
    , system
    , ...
    }: {
      packages.push_notifications_service_provider_test_happ = inputs.tnesh-stack.outputs.builders.${system}.happ {
        happManifest = ./happ.yaml;

        dnas = {
          # Include here the DNA packages for this hApp, e.g.:
          # my_dna = inputs'.some_input.packages.my_dna;
          # This overrides all the "bundled" properties for the hApp manifest 
          push_notifications_service_provider_test = self'.packages.push_notifications_service_provider_test_dna;
        };
      };
    };
}
