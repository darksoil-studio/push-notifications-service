{ inputs, ... }: {
  flake = {
    nixosConfigurations = let
      push_notifications_service_provider =
        inputs.self.outputs.packages."x86_64-linux".push-notifications-service-provider;
      push_notifications_service_provider_module = {
        systemd.services.push_notifications_service_provider = {
          enable = true;
          path = [ push_notifications_service_provider ];
          wantedBy = [ "multi-user.target" ];
          serviceConfig = {
            ExecStart =
              "${push_notifications_service_provider}/bin/push-notifications-service-provider --data-dir /root/push-notifications-service-provider";
            RuntimeMaxSec = "3600"; # Restart every hour

            Restart = "always";
            RestartSec = 1;
          };
        };
        system.stateVersion = "25.05";
        garnix.server.enable = true;
        garnix.server.persistence.enable = true;
      };

    in {
      push_notifications_service_provider1 = inputs.nixpkgs.lib.nixosSystem {
        system = "x86_64-linux";
        modules = [
          inputs.garnix-lib.nixosModules.garnix
          {
            garnix.server.persistence.name =
              "push-notifications-service-provider1";
          }
          push_notifications_service_provider_module
        ];
      };
      push_notifications_service_provider2 = inputs.nixpkgs.lib.nixosSystem {
        system = "x86_64-linux";
        modules = [
          inputs.garnix-lib.nixosModules.garnix
          {
            garnix.server.persistence.name =
              "push-notifications-service-provider2";
          }
          push_notifications_service_provider_module
        ];
      };
    };
  };
}

