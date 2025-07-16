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

  push-notifications-service-provider =
    inputs.self.outputs.packages."x86_64-linux".push-notifications-service-provider;

  push-notifications-service-provider-module = {
    systemd.services.push-notifications-service-provider = {
      enable = true;
      path = [ push-notifications-service-provider ];
      wantedBy = [ "multi-user.target" ];
      serviceConfig = {
        ExecStart =
          "${push-notifications-service-provider}/bin/push-notifications-service-provider --data-dir /root/push-notifications-service-provider";
        RuntimeMaxSec = "3600"; # Restart every hour

        Restart = "always";
        RestartSec = 1;
      };
    };
  };

in {
  flake = {

    nixosConfigurations = {
      push-notifications-service-provider1 = inputs.nixpkgs.lib.nixosSystem {
        system = "x86_64-linux";
        modules = [
          inputs.garnix-lib.nixosModules.garnix
          sshModule
          push-notifications-service-provider-module
          {
            garnix.server.persistence.name =
              "push-notifications-service-provider1";
            system.stateVersion = "25.05";
            garnix.server.enable = true;
            garnix.server.persistence.enable = true;
          }
        ];
      };
      push-notifications-service-provider2 = inputs.nixpkgs.lib.nixosSystem {
        system = "x86_64-linux";
        modules = [
          inputs.garnix-lib.nixosModules.garnix
          sshModule
          push-notifications-service-provider-module
          {
            garnix.server.persistence.name =
              "push-notifications-service-provider2";
            system.stateVersion = "25.05";
            garnix.server.enable = true;
            garnix.server.persistence.enable = true;
          }
        ];
      };
    };
  };
}

