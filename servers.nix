{ inputs, ... }:
let
  sshPubKeys = {
    guillem =
      "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIDTE+RwRfcG3UNTOZwGmQOKd5R+9jN0adH4BIaZvmWjO guillem.cordoba@gmail.com";
  };
  sshModule = {
    users.users.root.openssh.authorizedKeys.keys =
      builtins.attrValues sshPubKeys;
    services.openssh.settings.PermitRootLogin = "without-password";
  };

  push_notifications_service_provider =
    inputs.self.outputs.packages."x86_64-linux".push-notifications-service-provider;

  push_notifications_service_provider_module = {
    systemd.services.push_notifications_service_provider1 = {
      enable = true;
      path = [ push_notifications_service_provider ];
      wantedBy = [ "multi-user.target" ];
      serviceConfig = {
        ExecStart =
          "${push_notifications_service_provider}/bin/push-notifications-service-provider --data-dir /root/push-notifications-service-provider1";
        RuntimeMaxSec = "3600"; # Restart every hour

        Restart = "always";
        RestartSec = 1;
      };
    };
    systemd.services.push_notifications_service_provider2 = {
      enable = true;
      path = [ push_notifications_service_provider ];
      wantedBy = [ "multi-user.target" ];
      serviceConfig = {
        ExecStart =
          "${push_notifications_service_provider}/bin/push-notifications-service-provider --data-dir /root/push-notifications-service-provider2";
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
  flake = {

    nixosConfigurations = {
      push-notifications-service-provider1 = inputs.nixpkgs.lib.nixosSystem {
        system = "x86_64-linux";
        modules = [
          inputs.garnix-lib.nixosModules.garnix
          sshModule
          {
            garnix.server.persistence.name =
              "push-notifications-service-provider1";
          }
          push_notifications_service_provider_module
        ];
      };
      push-notifications-service-provider2 = inputs.nixpkgs.lib.nixosSystem {
        system = "x86_64-linux";
        modules = [
          inputs.garnix-lib.nixosModules.garnix
          sshModule
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

