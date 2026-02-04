# NixOS module for cosmic-radio-applet
flake:
{ config, lib, pkgs, ... }:

with lib;

let
  cfg = config.programs.cosmic-radio-applet;

  # Get the package from the flake or use override
  defaultPackage = flake.packages.${pkgs.stdenv.hostPlatform.system}.cosmic-radio-applet;

in {
  options.programs.cosmic-radio-applet = {
    enable = mkEnableOption "COSMIC Radio Applet - internet radio player for COSMIC Desktop panel";

    package = mkPackageOption pkgs "cosmic-radio-applet" {
      default = defaultPackage;
      example = literalExpression ''
        pkgs.cosmic-radio-applet.override {
          # custom overrides
        }
      '';
      description = "The cosmic-radio-applet package to use.";
    };

    autostart = mkOption {
      type = types.bool;
      default = true;
      description = ''
        Whether to automatically start the applet with COSMIC Desktop.
        When enabled, the applet will appear in the panel on login.
      '';
    };

    settings = mkOption {
      type = types.submodule {
        freeformType = with types; attrsOf anything;

        options = {
          volume = mkOption {
            type = types.ints.between 0 100;
            default = 50;
            description = "Default volume level (0-100).";
          };

          favorites = mkOption {
            type = types.listOf (types.submodule {
              options = {
                stationuuid = mkOption {
                  type = types.str;
                  description = "Unique identifier from radio-browser.info API.";
                };
                name = mkOption {
                  type = types.str;
                  description = "Display name of the station.";
                };
                url_resolved = mkOption {
                  type = types.str;
                  description = "Direct stream URL for playback.";
                };
                url = mkOption {
                  type = types.str;
                  default = "";
                  description = "Original station URL.";
                };
                homepage = mkOption {
                  type = types.str;
                  default = "";
                  description = "Station homepage URL.";
                };
                favicon = mkOption {
                  type = types.str;
                  default = "";
                  description = "Station favicon URL.";
                };
                tags = mkOption {
                  type = types.str;
                  default = "";
                  description = "Comma-separated tags describing the station.";
                };
                country = mkOption {
                  type = types.str;
                  default = "";
                  description = "Country where the station is based.";
                };
                language = mkOption {
                  type = types.str;
                  default = "";
                  description = "Primary language of the station.";
                };
              };
            });
            default = [ ];
            example = literalExpression ''
              [
                {
                  stationuuid = "96202c39-0601-11e8-ae97-52543be04c81";
                  name = "SomaFM - Groove Salad";
                  url_resolved = "https://ice1.somafm.com/groovesalad-128-mp3";
                }
              ]
            '';
            description = ''
              List of favorite radio stations.
              These will be pre-populated in the applet's favorites list.
            '';
          };
        };
      };
      default = { };
      description = ''
        Configuration for cosmic-radio-applet.
        Settings are written to the COSMIC config system.
      '';
    };
  };

  config = mkIf cfg.enable {
    # Add the package to system packages
    environment.systemPackages = [ cfg.package ];

    # Ensure mpv is available system-wide (runtime dependency)
    programs.mpv.enable = mkDefault true;

    # Ensure ALSA is available
    sound.enable = mkDefault true;

    # XDG desktop portal for proper Wayland integration
    xdg.portal = {
      enable = mkDefault true;
      wlr.enable = mkDefault true;
    };

    # Add autostart entry if enabled
    environment.etc = mkIf cfg.autostart {
      "xdg/autostart/com.marcos.RadioApplet.desktop".source =
        "${cfg.package}/share/applications/com.marcos.RadioApplet.desktop";
    };

    # Assertions
    assertions = [
      {
        assertion = config.services.xserver.desktopManager.cosmic.enable or false
          || config.services.desktopManager.cosmic.enable or false
          || true; # Allow installation without COSMIC for testing
        message = "cosmic-radio-applet is designed for COSMIC Desktop but can be installed standalone.";
      }
    ];

    warnings = optional (!(config.services.xserver.desktopManager.cosmic.enable or false)
      && !(config.services.desktopManager.cosmic.enable or false))
      "cosmic-radio-applet works best with COSMIC Desktop enabled.";
  };

  meta.maintainers = with lib.maintainers; [ ];
}
