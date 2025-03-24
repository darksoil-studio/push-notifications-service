{ inputs, ... }:

{
  perSystem = { inputs', system, self', ... }: {
    packages.push_notifications_service =
      inputs.tnesh-stack.outputs.builders.${system}.rustZome {
        workspacePath = inputs.self.outPath;
        crateCargoToml = ./Cargo.toml;
      };

  };
}

