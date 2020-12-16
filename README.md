# ![](https://i.imgur.com/FpdXqiT.png) Snekdown - More than just Markdown ![](https://img.shields.io/discord/729250668162056313)


This projects goal is to implement a fast markdown parser with an extended syntax fitted
for my needs.

## Installation

You need a working rust installation, for example by using [rustup](http://rustup.rs).

```sh
cargo install snekdown
```

With pdf rendering

```sh
cargo install snekdown --features pdf
```

## Usage

```
snekdown 0.30.5

USAGE:
    snekdown <SUBCOMMAND>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

SUBCOMMANDS:
    clear-cache    Clears the cache directory
    help           Prints this message or the help of the given subcommand(s)
    render         Parse and render the document
    watch          Watch the document and its imports and render on change
```

### Rendering

```
Parse and render the document

USAGE:
    snekdown render [OPTIONS] <input> <output>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -f, --format <format>    the output format [default: html]

ARGS:
    <input>     Path to the input file
    <output>    Path for the output file

```

### Watching

```
Watch the document and its imports and render on change

USAGE:
    snekdown watch [OPTIONS] <input> <output>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
        --debounce <debounce>    The amount of time in milliseconds to wait after changes before rendering [default:
                                 500]
    -f, --format <format>        the output format [default: html]

ARGS:
    <input>     Path to the input file
    <output>    Path for the output file
```


## Syntax

### Images

```md
Simple Syntax
!(url)

Extended syntax with a description
![description](url)

Extended syntax with metadata to specify the size
![description](url)[metadata]

Extended syntax with metadata and no description
!(url)[metadata]
```

When generating the html file the images are base64 embedded. To turn off this behaviour
set the config parameter `embed-external` to `false`.

### Quotes

```md
Simple (default) Syntax
> This is a quote

Multiline
> This is a 
> Multiline Quote

Quote with metadata (e.g. Author)
[author=Trivernis year=2020 display='{{author}} - {{year}}']> This is a quote with metadata
```


### Imports

Imports can be used to import a different document to be attached to the main document.
Imports are parsed via multithreading.

```md
<[path]

<[document.md]

<[style.css][type=stylesheet]
```

The parser differentiates four different types of imported files.

- `document`            - The default import which is just another snekdown document
- `stylesheet`          - CSS Stylesheets that are inclued when rendering
- `bibliography`        - A file including bibliography
- `config`/`manifest`   - A config file that contains metadata

If no type is provided the parser guesses the type of file from the extension.

### Tables

Tables MUST start with a pipe character `|`

```md
Standalone header:
| header | header | header

Header with rows
| header | header | header
|--------|--------|-------
| row    | row    | row
```

### Placeholders

Placeholders can be used to insert special elements in a specific place.
Placeholders are always case insensitive.

```md
Insert the table of contents
[[TOC]]

Insert the current date
[[date]]

Insert the current time
[[time]]
```

### Metadata

Additional metadata can be provided for some elements.

```md
String value
[key = value]

String value
[key = "String value"]

Integer value
[key = 123]

Float value
[key = 1.23]

Boolean
[key] 

Boolean
[key = false]

Placeholder
[key = [[placeholder]]]
```

Metadata can also be defined in a separate toml file with simple key-value pairs.
Example:

```toml
# bibliography.bib.toml
author = "Snek"
published = "2020"
test-key = ["test value", "test value 2"]

# those files won't get imported
ignored-imports = ["style.css"]        

# stylesheets that should be included
included-stylesheets = ["style2.css"] 

# other metadata files that should be included
included-configs = []

# bibliography that should be included
included-bibliography = ["mybib.toml"]

# glossary that sould be included      
included-glossary = ["myglossary.toml"]     

# if external sources (images, stylesheets, MathJax)
# should be embedded into the document (default: true)
embed-external = true

# If SmartArrows should be used (default: true)
smart-arrows = true

# Includes a MathJax script tag in the document to render MathML in chromium.
# (default: true)
include-math-jax = true


### Image processing options ###

# Force convert images to the specified format.
# Supported formats are png, jpeg, gif, bmp, (ico needs size <= 256), avif, pnm
# (default: keep original)
image-format = "jpg"

# the max width for the images.
# if an image is larger than that it get's resized.
# (default: none)
image-max-width = 700

# the max width for the images.
# if an image is larger than that it get's resized.
# (default: none)
image-max-height = 800


### PDF Options - needs the pdf feature enabled ###

# If the header and footer of the pdf should be displayed (default: true)
pdf-display-header-footer = true

# PDF header template of each page (default: empty)
pdf-header-template = "<div><span class='title'></span></div>"

# PDF footer template of each page (default: see chromium_pdf assets)
pdf-footer-template = "<div><span class='pageNumber'></span></div>"

# Top margin of the pdf. Should be between 0 and 1. (default: 1.0)
pdf-margin-top = 1

# Bottom margin of the pdf. Should be between 0 and 1. (default: 1.0)
pdf-margin-bottom = 1

# Left margin of the pdf. Should be between 0 and 1.
pdf-margin-left = 0

# Right margin of the pdf. Should be between 0 and 1.
pdf-margin-right = 0

# Page height of the pdf
pdf-page-height = 100

# Page width of the pdf
pdf-page-width = 80

# The scale at which the website is rendered into pdf.
pdf-page-scale = 1.0
```

The `[Section]` keys are not relevant as the structure gets flattened before the values are read.


#### Usage

```
Hide a section (including subsections) in the TOC
#[toc-hidden] Section

Set the size of an image
!(url)[width = 42%, height=auto, brightness=10, contrast=1.2, huerotate=180, invert, grayscale]

Set the source of a quote
[author=Me date=[[date]] display="{{author}} - {{date}}"]> It's me

Set options for placeholders
[[toc]][ordered]
```

### Centered Text

```
|| These two lines
|| are centered
```

### Inline

```md
*Italic*
**Bold**
~~Striked~~
_Underlined_
^Superscript^
`Monospace`
:Emoji:
§[#0C0]Colored text§[] §[red] red §[]
```

## Bibliography

Bibliography entries can be defined and referenced anywhere in the document.

Definition:
```md
[SD_BOOK]:[type=book, author=Snek, title = "Snekdown Book" date="20.08.2020", publisher=Snek]
[SD_GITHUB]: https://github.com/trivernis/snekdown
```

Usage:
```
There is a book about snekdown[^book] and a github repo[^github].
```

Entries can also be defined in a separate toml file with the following data layout:

```toml
# snekdown.toml
[BIB_KEY]
key = "value"

[SD_BOOK]
type = "book"
author = "Snek"
title = "Snekdown Book"
date = "20.08.2020"
publisher = "Snek"

[SD_GITHUB]
type = "website"
url = "https://github.com/trivernis/snekdown"
```

The valid types for entries and required fields can be found on in the [bibliographix README](https://github.com/Trivernis/bibliographix#bibliography-types-and-fields).

Bibliography entries are not rendered. To render a list of used bibliography insert the
`bib` placeholder at the place you want it to be rendered.


## Glossary

Glossary entries are to be defined in a `glossary.toml` file or any other toml file
that is imported as type `glossary`.
The definition of glossary entries has to follow the following structure

```toml
[SHORT]
long = "Long Form"
description = "The description of the entry"

# Example
[HTML]
long = "Hypertext Markup Language"
description = "The markup language of the web"
```

Those glossary entries can be referenced in the snekdown file as follows:

```md
~HTML is widely used for websites.
The format ~HTML is not considered a programming language by some definitions.

~~HTML
```

The first occurence of the glossary entry (`~HTML`) always uses the long form.
The second will always be the short form. The long form can be enforced by using two
(`~~HTML`) tildes.

## Math

Snekdown allows the embedding of [AsciiMath](http://asciimath.org/):
The AsciiMath parser is provided in the [asciimath-rs](https://github.com/Trivernis/asciimath-rs) crate

```
inline math $$ a^2 + b^2 = c^2 $$

Block Math
$$$
A = [[1, 2],[3,4]]
$$$
```

The expression get's converted into MathML which is then converted by MathJax when loaded in
the browser.

## Smart Arrows

Snekdown automatically renders the sequences `-->`, `==>`, `<--`, `<==`, `<-->`, `<==>` as
their respective unicode arrows (similar to [markdown-it-smartarrows](https://github.com/adam-p/markdown-it-smartarrows)).
This behavior can be turned off by setting the config parameter `smart-arrows` to `false`
(the config needs to be imported before the arrows are used for that to work).


## Roadmap

The end goal is to have a markup language with features similar to LaTeX.

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
- [ ] Custom Elements via templates (50%)
- [ ] Cross References
- [ ] Figures
- [ ] EPUB Rendering
- [ ] Text sizes
- [ ] Title pages