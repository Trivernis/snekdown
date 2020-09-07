# ![](https://i.imgur.com/FpdXqiT.png) Snekdown - More than just Markdown ![](https://img.shields.io/discord/729250668162056313)


This projects goal is to implement a fast markdown parser with an extended syntax fitted
for my needs.

## Usage

```
USAGE:
    snekdown [FLAGS] [OPTIONS] <input> <output> [SUBCOMMAND]

FLAGS:
    -h, --help        Prints help information
        --no-cache    Don't use the cache
    -V, --version     Prints version information

OPTIONS:
    -f, --format <format>    the output format [default: html]

ARGS:
    <input>     Path to the input file
    <output>    Path for the output file

SUBCOMMANDS:
    help      Prints this message or the help of the given subcommand(s)
    render    Default. Parse and render the document
    watch     Watch the document and its imports and render on change
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
[Meta]
author = "Snek"
published = "2020"
test-key = ["test value", "test value 2"]

[imports]
ignored-imports = ["style.css"]         # those files won't get imported
included-stylesheets = ["style2.css"]   # stylesheets that should be included
included-configs = []                   # other metadata files that should be included
included-bibliography = ["mybib.toml"]  # bibliography that should be included
included-glossary = ["myglossary.toml"] #glossary that sould be included
```

The `[Section]` keys are not relevant as the structure gets flattened before the values are read.


#### Usage

```
Hide a section (including subsections) in the TOC
#[toc-hidden] Section

Set the size of an image
!(url)[width = 42% height=auto]

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
~Striked~
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

## Roadmap

The end goal is to have a markup language with features similar to LaTeX.

- [x] Checkboxes
- [x] Emojis (\:emoji:)
- [x] Colors
- [x] Watching and rendering on change
- [x] Metadata files
- [x] Bibliography
- [x] Math
- [ ] Text sizes
- [ ] Title pages
- [x] Glossary
- [ ] Cross References
- [ ] Figures
- [ ] EPUB Rendering (PDF is too hard)
- [ ] Custom Elements via templates (50%)
- [x] Custom Stylesheets
- [ ] Smart arrows
