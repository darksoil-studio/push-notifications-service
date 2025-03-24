{ inputs, ... }:

{
  perSystem = { inputs', system, self', ... }: {
    packages.push_notifications_service_providers_manager =
      inputs.tnesh-stack.outputs.builders.${system}.rustZome {
        workspacePath = inputs.self.outPath;
        crateCargoToml = ./Cargo.toml;
      };

  };
}

