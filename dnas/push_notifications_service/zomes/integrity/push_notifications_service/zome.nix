{ inputs, ... }:

{
  perSystem = { inputs', system, ... }: {
    packages.push_notifications_service_integrity =
      inputs.tnesh-stack.outputs.builders.${system}.rustZome {
        workspacePath = inputs.self.outPath;
        crateCargoToml = ./Cargo.toml;
      };
  };
}

