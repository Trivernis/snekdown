<p align="center">
<img src="https://i.imgur.com/FpdXqiT.png">
</p>
<h1 align="center">Snekdown</h1>
<p align="center">
<i>More than just Markdown</i>
</p>
<p align="center">
    <a href="https://github.com/Trivernis/snekdown/actions">
        <img src="https://img.shields.io/github/workflow/status/trivernis/snekdown/Build%20and%20Test/main?style=for-the-badge">
    </a>
    <a href="https://crates.io/crates/snekdown">
        <img src="https://img.shields.io/crates/v/snekdown?style=for-the-badge">
    </a>
    <a href="https://aur.archlinux.org/packages/snekdown">
        <img src="https://img.shields.io/aur/version/snekdown?style=for-the-badge">
    </a>
    <a href="https://discord.gg/vGAXW9nxUv">
        <img src="https://img.shields.io/discord/729250668162056313?style=for-the-badge">
    </a>
<br/>
<br/>
<a href="https://trivernis.net/snekdown/">Documentation</a> |
<a href="https://github.com/Trivernis/snekdown/releases">Releases</a>
</p>

- - -

## Description

This projects goal is to implement a fast markdown parser with an extended syntax fitted
for my needs.

## Core Features

- Imports
- Bibliography & Glossary
- AsciiMath
- Placeholders
- Advanced Images


## Installation

### Binaries

You can download prebuilt binaries on the [Releases](https://github.com/Trivernis/snekdown/releases) Page.


### Arch Linux

Snekdown is available in [the AUR](https://aur.archlinux.org/packages/snekdown).


### Cargo

You need a working rust installation, for example by using [rustup](http://rustup.rs).

```sh
cargo install snekdown
```

With pdf rendering

```sh
cargo install snekdown --features pdf
```


## Usage

Use `snekdown help` and `snekdown <subcommand> --help` for more information.

### Rendering

`snekdown render <input> <output>`

### Watching

`snekdown watch <input> <output>`


## Editors

I've created a [VisualStudio Code extension](https://marketplace.visualstudio.com/items?itemName=trivernis.snekdown) for Snekdown.
This extension provides a preview of snekdown files, exports and other commands similar to the
cli. The source code can be found [here](https://github.com/Trivernis/snekdown-vscode-extension).


## Roadmap

The end goal is to have a markup language with features similar to LaTeX.

### Short Term

- [x] Checkboxes
- [x] Emojis (\:emoji:)
- [x] Colors
- [x] Watching and rendering on change
- [x] Metadata files
- [x] Bibliography
- [x] Math
- [x] Glossary
- [x] Chromium based pdf rendering
- [x] Custom Stylesheets
- [x] Smart arrows
- [ ] Cross References
- [ ] Figures
- [ ] EPUB Rendering
- [ ] Text sizes
- [ ] Title pages


### Long Term

- Rewrite of the whole parsing process
- Custom Elements via templates


## License

This project is licensed under GPL 3.0. See LICENSE for more information.