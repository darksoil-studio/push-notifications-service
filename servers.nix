{ inputs, ... }:
let

  sshPubKeys = {
    guillem =
      "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIDTE+RwRfcG3UNTOZwGmQOKd5R+9jN0adH4BIaZvmWjO guillem.cordoba@gmail.com";
    guillemslaptop =
      "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIO8DVpvRgQ90MyMyiuNdvyMNAio9n2o/+57MyhZS2A5A guillem.cordoba@gmail.com";
  };

  sshModule = {
    users.users.root.openssh.authorizedKeys.keys =
      builtins.attrValues sshPubKeys;
    services.openssh.settings.PermitRootLogin = "without-password";
    services.openssh.enable = true;
  };

  bootstrapServerUrl =
    "https://bootstrap.kitsune-v0-1.kitsune.darksoil-studio.garnix.me";

  push-notifications-service-provider =
    inputs.self.outputs.packages."x86_64-linux".push-notifications-service-provider;

  push-notifications-service-provider-module = {
    systemd.services.push-notifications-service-provider = {
      enable = true;
      path = [ push-notifications-service-provider ];
      wantedBy = [ "multi-user.target" ];
      after = [ "network-online.target" ];
      wants = [ "network-online.target" ];
      serviceConfig = {
        ExecStart =
          "${push-notifications-service-provider}/bin/push-notifications-service-provider --data-dir /root/push-notifications-service-provider --bootstrap-url ${bootstrapServerUrl} --admin-port 8080";
        RuntimeMaxSec = "3600"; # Restart every hour
        Restart = "always";
      };
    };
    # Set limits for systemd units (not systemd itself).
    #
    # From `man 5 systemd-system.conf`:
    # "DefaultLimitNOFILE= defaults to 1024:524288"
    systemd.extraConfig = ''
      DefaultLimitNOFILE=8192:524288
    '';
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
            system.stateVersion = "25.05";
            garnix.server.enable = true;
            garnix.server.persistence.enable = true;
            garnix.server.persistence.name =
              "push-notifications-service-provider-v0-502-1";
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
              "push-notifications-service-provider-v0-502-2";
            system.stateVersion = "25.05";
            garnix.server.enable = true;
            garnix.server.persistence.enable = true;
          }
        ];
      };
    };
  };

}

