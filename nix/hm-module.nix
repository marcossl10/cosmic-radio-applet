# Home Manager module for cosmic-radio-applet
flake:
{ config, lib, pkgs, ... }:

with lib;

let
  cfg = config.programs.cosmic-radio-applet;

  # Get the package from the flake or use override
  defaultPackage = flake.packages.${pkgs.stdenv.hostPlatform.system}.cosmic-radio-applet;

  # Generate cosmic config JSON
  configJson = pkgs.writeText "cosmic-radio-applet-config.json" (builtins.toJSON {
    favorites = cfg.settings.favorites;
    volume = cfg.settings.volume;
  });

in {
  options.programs.cosmic-radio-applet = {
    enable = mkEnableOption "COSMIC Radio Applet - internet radio player for COSMIC Desktop panel";

    package = mkOption {
      type = types.package;
      default = defaultPackage;
      defaultText = literalExpression "flake.packages.\${pkgs.stdenv.hostPlatform.system}.cosmic-radio-applet";
      description = "The cosmic-radio-applet package to use.";
    };

    autostart = mkOption {
      type = types.bool;
      default = true;
      description = ''
        Whether to automatically start the applet with COSMIC Desktop.
        When enabled, adds the applet to XDG autostart.
      '';
    };

    settings = {
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
              description = "Comma-separated tags.";
            };
            country = mkOption {
              type = types.str;
              default = "";
              description = "Country of origin.";
            };
            language = mkOption {
              type = types.str;
              default = "";
              description = "Primary language.";
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
            {
              stationuuid = "9617a958-0601-11e8-ae97-52543be04c81";
              name = "Jazz24";
              url_resolved = "https://live.wostreaming.net/direct/ppm-jazz24aac-ibc1";
            }
          ]
        '';
        description = ''
          List of favorite radio stations to pre-populate.
          Stations can be found via the radio-browser.info API.
        '';
      };
    };
  };

  config = mkIf cfg.enable {
    # Add package to user environment
    home.packages = [ cfg.package ];

    # Ensure mpv is available (runtime dependency)
    programs.mpv.enable = mkDefault true;

    # Add autostart entry
    xdg.configFile = mkIf cfg.autostart {
      "autostart/com.marcos.RadioApplet.desktop".source =
        "${cfg.package}/share/applications/com.marcos.RadioApplet.desktop";
    };

    # Write COSMIC config if favorites are defined
    # COSMIC stores config at ~/.config/cosmic/com.marcos.RadioApplet/v9/
    xdg.configFile = mkIf (cfg.settings.favorites != [ ] || cfg.settings.volume != 50) {
      "cosmic/com.marcos.RadioApplet/v9/config.json" = {
        text = builtins.toJSON {
          favorites = map (station: {
            inherit (station) stationuuid name url url_resolved homepage favicon tags country language;
          }) cfg.settings.favorites;
          volume = cfg.settings.volume;
        };
      };
    };

    # Verify mpv is in PATH
    warnings = optional (!config.programs.mpv.enable)
      "cosmic-radio-applet requires mpv for audio playback. Consider enabling programs.mpv.";
  };

  meta.maintainers = with lib.maintainers; [ ];
}
