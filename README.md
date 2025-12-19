# COSMIC Radio Applet

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
### 🚀 Installation

#### Prerequisites

Make sure you have `mpv` installed on your system:

```bash
# Arch Linux
sudo pacman -S mpv

# Fedora
sudo dnf install mpv

# Ubuntu/Pop!_OS
sudo apt install mpv
```

#### Compile and Install

Clone the repository and use `just` to install:

```bash
git clone https://github.com/marcossl10/cosmic-radio-applet.git
cd cosmic-radio-applet
sudo just install
```

### � License

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
- **Amplificação e Normalização**: Suporte a volume de até 200% e normalização dinâmica de áudio.

---
Developed by [marcossl10](https://github.com/marcossl10).
