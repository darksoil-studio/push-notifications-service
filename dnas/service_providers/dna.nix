{ inputs, ... }:

{
  imports = (map (m: "${./.}/zomes/coordinator/${m}/zome.nix")
    (builtins.attrNames (builtins.readDir ./zomes/coordinator)));

  perSystem = { inputs', self', lib, system, ... }: {
    packages.services_dna_with_push_notifications_gateway =
      inputs.service-providers.outputs.builders.${system}.services_dna_with_gateway {
        gatewayZome = self'.packages.push_notifications_gateway;
      };
  };
}

