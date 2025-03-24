{ inputs, ... }:

{
  imports = (map (m: "${./.}/zomes/coordinator/${m}/zome.nix")
    (builtins.attrNames (builtins.readDir ./zomes/coordinator)));

  perSystem = { inputs', self', lib, system, ... }: {
    packages.service_providers_dna =
      inputs.service-providers.outputs.builders.${system}.service_providers_dna {
        gatewayZome = self'.packages.push_notifications_gateway;
      };
  };
}

