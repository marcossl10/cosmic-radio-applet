# COSMIC Radio Applet

[![CI](https://github.com/marcossl10/cosmic-radio-applet/workflows/CI/badge.svg)](https://github.com/marcossl10/cosmic-radio-applet/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

[English](#english) | [Português](#português)

---

## English

A modern online radio player integrated into the COSMIC Desktop panel, developed exclusively for the COSMIC ecosystem using Rust and libcosmic.

<img src="resources/banner.svg" width="600" alt="Banner">

### ✨ Features

- **Global Search**: Access thousands of radio stations worldwide via the `radio-browser.info` API.
- **Native Interface**: Design perfectly integrated with COSMIC Desktop, following the system's visual guidelines.
- **Interactive Playback**: Click on a station to Play/Pause (Stop).
- **Favorites List**: Save your preferred stations for quick access.
- **High-Quality Audio**: Uses `mpv` as the playback backend, ensuring stability and low resource consumption.
- **Volume Control**: Interactive slider with live adjustment and visual feedback (muted/low/medium/high icons).
- **Keyboard Shortcuts**: Quick controls without mouse - Space (play/pause), Arrow keys (volume), Escape (close).
- **Internationalization**: Multi-language support with Fluent localization system.
- **Security Hardened**: URL validation, private IP blocking, and response size limits.

### ⌨️ Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `Space` | Toggle play/pause |
| `↑` | Volume up (+5%) |
| `↓` | Volume down (-5%) |
| `Esc` | Close popup |

### 🚀 Installation

#### Prerequisites

Ensure `alsa-utils` is installed on your system:

```bash
# Debian/Ubuntu
sudo apt install alsa-utils

# Arch Linux
sudo pacman -S alsa-utils

# Fedora
sudo dnf install alsa-utils

# OpenSUSE
sudo zypper install alsa-utils
```

Make sure you have `mpv` installed on your system:

```bash
# Arch Linux
sudo pacman -S mpv

# Fedora
sudo dnf install mpv

# Ubuntu/Pop!_OS
sudo apt install mpv
```

#### System Dependencies for Building

To compile the project, you'll need the following development packages:

**For Debian/Ubuntu/Linux Mint/Pop!_OS:**
```bash
sudo apt update
sudo apt install -y pkg-config libxkbcommon-dev libwayland-dev libssl-dev libasound2-dev
```

**For Fedora/RHEL/CentOS:**
```bash
sudo dnf install pkgconf-pkg-config libxkbcommon-devel wayland-devel openssl-devel alsa-lib-devel
```

**For Arch Linux/Manjaro:**
```bash
sudo pacman -S pkgconf libxkbcommon wayland openssl alsa-lib
```

#### Compile and Install

Clone the repository and use `just` to install:

```bash
git clone https://github.com/marcossl10/cosmic-radio-applet.git
cd cosmic-radio-applet
sudo just install
```

#### NixOS Installation

This project includes a Nix flake with NixOS and Home Manager modules.

**Using Nix Flakes:**
```bash
# Build and run directly
nix run github:marcossl10/cosmic-radio-applet

# Or build the package
nix build github:marcossl10/cosmic-radio-applet
```

**NixOS Module:**
```nix
{
  inputs.cosmic-radio-applet.url = "github:marcossl10/cosmic-radio-applet";

  outputs = { nixpkgs, cosmic-radio-applet, ... }: {
    nixosConfigurations.yourhost = nixpkgs.lib.nixosSystem {
      modules = [
        cosmic-radio-applet.nixosModules.cosmic-radio-applet
        {
          programs.cosmic-radio-applet = {
            enable = true;
            settings.volume = 75;
          };
        }
      ];
    };
  };
}
```

**Home Manager Module:**
```nix
{
  imports = [ cosmic-radio-applet.homeManagerModules.cosmic-radio-applet ];

  programs.cosmic-radio-applet = {
    enable = true;
    autostart = true;
    settings = {
      volume = 50;
      favorites = [{
        stationuuid = "96202c39-0601-11e8-ae97-52543be04c81";
        name = "SomaFM - Groove Salad";
        url_resolved = "https://ice1.somafm.com/groovesalad-128-mp3";
      }];
    };
  };
}
```

### 🛠️ Development

#### Building from Source

```bash
git clone https://github.com/marcossl10/cosmic-radio-applet.git
cd cosmic-radio-applet
cargo build --release
```

#### Running Tests

```bash
cargo test
```

#### Code Quality

```bash
cargo fmt --check    # Check formatting
cargo clippy         # Run linter
```

#### CI/CD

This project uses GitHub Actions for continuous integration with:
- Format and lint checks
- Debug and release builds
- Unit tests (57 tests)
- Nix flake builds
- Security audit (`cargo-audit`)
- Dependency policy checks (`cargo-deny`)

### 🔒 Security

- **URL Validation**: Only `http://` and `https://` schemes are allowed
- **Private IP Blocking**: Localhost, 127.0.0.1, and private ranges (192.168.x.x, 10.x.x.x, 172.16.x.x) are blocked
- **Response Size Limits**: API responses are limited to 1MB to prevent memory exhaustion
- **Secure Defaults**: All external inputs are validated before processing

### 📄 License

This project is under the [MIT](LICENSE) license.

---

## Português

Um player de rádio online moderno e integrado ao painel do COSMIC Desktop, desenvolvido exclusivamente para o ecossistema COSMIC usando Rust e libcosmic.

<img src="resources/banner.svg" width="600" alt="Banner">

### ✨ Funcionalidades

- **Busca Global**: Acesse milhares de estações de rádio de todo o mundo via API `radio-browser.info`.
- **Interface Nativa**: Design perfeitamente integrado ao COSMIC Desktop.
- **Controle de Reprodução**: Clique na rádio para dar Play/Pause (Stop).
- **Lista de Favoritos**: Salve suas estações preferidas.
- **Áudio de Alta Qualidade**: Utiliza o `mpv` como backend de reprodução.
- **Controle de Volume**: Slider interativo com ajuste em tempo real e feedback visual (ícones mudo/baixo/médio/alto).
- **Atalhos de Teclado**: Controles rápidos sem mouse - Espaço (play/pause), Setas (volume), Escape (fechar).
- **Internacionalização**: Suporte multi-idioma com sistema de localização Fluent.
- **Segurança Reforçada**: Validação de URLs, bloqueio de IPs privados e limites de resposta.

### ⌨️ Atalhos de Teclado

| Tecla | Ação |
|-------|------|
| `Espaço` | Alternar play/pause |
| `↑` | Aumentar volume (+5%) |
| `↓` | Diminuir volume (-5%) |
| `Esc` | Fechar popup |

### 🚀 Instalação NixOS

Este projeto inclui um flake Nix com módulos NixOS e Home Manager.

**Usando Nix Flakes:**
```bash
# Compilar e executar diretamente
nix run github:marcossl10/cosmic-radio-applet

# Ou compilar o pacote
nix build github:marcossl10/cosmic-radio-applet
```

**Módulo NixOS:**
```nix
{
  inputs.cosmic-radio-applet.url = "github:marcossl10/cosmic-radio-applet";

  outputs = { nixpkgs, cosmic-radio-applet, ... }: {
    nixosConfigurations.seuhost = nixpkgs.lib.nixosSystem {
      modules = [
        cosmic-radio-applet.nixosModules.cosmic-radio-applet
        {
          programs.cosmic-radio-applet = {
            enable = true;
            settings.volume = 75;
          };
        }
      ];
    };
  };
}
```

**Módulo Home Manager:**
```nix
{
  imports = [ cosmic-radio-applet.homeManagerModules.cosmic-radio-applet ];

  programs.cosmic-radio-applet = {
    enable = true;
    autostart = true;
    settings = {
      volume = 50;
      favorites = [{
        stationuuid = "96202c39-0601-11e8-ae97-52543be04c81";
        name = "SomaFM - Groove Salad";
        url_resolved = "https://ice1.somafm.com/groovesalad-128-mp3";
      }];
    };
  };
}
```

### 🛠️ Desenvolvimento

#### Compilando do Código Fonte

```bash
git clone https://github.com/marcossl10/cosmic-radio-applet.git
cd cosmic-radio-applet
cargo build --release
```

#### Executando Testes

```bash
cargo test
```

#### Qualidade de Código

```bash
cargo fmt --check    # Verificar formatação
cargo clippy         # Executar linter
```

#### CI/CD

Este projeto usa GitHub Actions para integração contínua com:
- Verificações de formato e lint
- Builds de debug e release
- Testes unitários (57 testes)
- Builds com Nix flake
- Auditoria de segurança (`cargo-audit`)
- Verificações de política de dependências (`cargo-deny`)

### 🔒 Segurança

- **Validação de URL**: Apenas esquemas `http://` e `https://` são permitidos
- **Bloqueio de IP Privado**: Localhost, 127.0.0.1 e faixas privadas (192.168.x.x, 10.x.x.x, 172.16.x.x) são bloqueados
- **Limites de Tamanho de Resposta**: Respostas da API são limitadas a 1MB para prevenir esgotamento de memória
- **Padrões Seguros**: Todas as entradas externas são validadas antes do processamento

### 📄 Licença

Este projeto está sob a licença [MIT](LICENSE).

---
Developed by [marcossl10](https://github.com/marcossl10).
