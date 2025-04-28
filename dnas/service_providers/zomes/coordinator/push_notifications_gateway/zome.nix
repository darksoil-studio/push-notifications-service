{ inputs, ... }:

{
  perSystem = { inputs', system, self', ... }: {
    packages.push_notifications_gateway =
      inputs.holochain-nix-builders.outputs.builders.${system}.rustZome {
        workspacePath = inputs.self.outPath;
        crateCargoToml = ./Cargo.toml;
      };
  };
}

