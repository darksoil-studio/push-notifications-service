{ inputs, ... }:

{
  perSystem = { inputs', system, ... }: {
    packages.push_notifications_service_integrity =
      inputs.holochain-nix-builders.outputs.builders.${system}.rustZome {
        workspacePath = inputs.self.outPath;
        crateCargoToml = ./Cargo.toml;
      };
  };
}

