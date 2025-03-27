{
  description = "Template for Holochain app development";

  inputs = {
    holonix.url = "github:holochain/holonix/main-0.4";
    crane.follows = "holonix/crane";
    nixpkgs.follows = "holonix/nixpkgs";
    flake-parts.follows = "holonix/flake-parts";

    tnesh-stack.url = "github:darksoil-studio/tnesh-stack/main-0.4";
    playground.url = "github:darksoil-studio/holochain-playground/main-0.4";

    service-providers.url = "/home/guillem/projects/darksoil/service-providers";
  };

  nixConfig = {
    extra-substituters = [
      "https://holochain-ci.cachix.org"
      "https://darksoil-studio.cachix.org"
    ];
    extra-trusted-public-keys = [
      "holochain-ci.cachix.org-1:5IUSkZc0aoRS53rfkvH9Kid40NpyjwCMCzwRTXy+QN8="
      "darksoil-studio.cachix.org-1:UEi+aujy44s41XL/pscLw37KEVpTEIn8N/kn7jO8rkc="
    ];
  };

  outputs = inputs:
    inputs.flake-parts.lib.mkFlake { inherit inputs; } {
      imports = [
        ./workdir/happ.nix
        ./crates/push_notifications_service_provider/default.nix
      ];

      systems = builtins.attrNames inputs.holonix.devShells;
      perSystem = { inputs', config, pkgs, system, ... }: {
        devShells.default = pkgs.mkShell {
          inputsFrom = [
            inputs'.tnesh-stack.devShells.synchronized-pnpm
            inputs'.tnesh-stack.devShells.holochainDev
            inputs'.holonix.devShells.default
          ];

          packages = [
            inputs'.tnesh-stack.packages.holochain
            inputs'.tnesh-stack.packages.hc-scaffold-zome
            inputs'.playground.packages.hc-playground
          ];
        };
        devShells.npm-ci = inputs'.tnesh-stack.devShells.synchronized-pnpm;

        # packages.scaffold = pkgs.symlinkJoin {
        #   name = "scaffold-remote-zome";
        #   paths = [ inputs'.tnesh-stack.packages.scaffold-remote-zome ];
        #   buildInputs = [ pkgs.makeWrapper ];
        #   postBuild = ''
        #     wrapProgram $out/bin/scaffold-remote-zome \
        #       --add-flags "push-notifications-service-provider-zome \
        #         --integrity-zome-name push_notifications_service_provider_integrity \
        #         --coordinator-zome-name push_notifications_service_provider \
        #         --remote-zome-git-url github:darksoil-studio/push-notifications-service-provider-zome \
        #         --remote-npm-package-name @darksoil-studio/push-notifications-service-provider-zome \
        #         --remote-zome-git-branch main-0.4 \
        #         --context-element push-notifications-service-provider-context \
        #         --context-element-import @darksoil-studio/push-notifications-service-provider-zome/dist/elements/push-notifications-service-provider-context.js" 
        #   '';
        # };
      };
    };
}
