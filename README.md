# :snake: Snekdown - More than just Markdown ![](https://img.shields.io/discord/729250668162056313)

This projects goal is to implement a fast markdown parser with an extended syntax fitted
for my needs.

## Usage

```
USAGE:
    snekdown [OPTIONS] <input> <output> [SUBCOMMAND]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

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
[![FOSSA Status](https://app.fossa.com/api/projects/git%2Bgithub.com%2FTrivernis%2Fsnekdown.svg?type=shield)](https://app.fossa.com/projects/git%2Bgithub.com%2FTrivernis%2Fsnekdown?ref=badge_shield)

Extended syntax with metadata to specify the size
![description](url)[metadata]

Extended syntax with metadata and no description
!(url)[metadata]
```


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
```


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

Formatting
[author = "The Great snek" date = [[date]] time = [[time]] display = "author - date at time"]
```

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
[book]:[author=Snek, title = "Snekdown Book"]
[github]: https://github.com/trivernis/snekdown
```

Usage:
```
There is a book about snekdown[^book] and a github repo[^github].
```

Bibliography entries are only shown when used in the document.

## Roadmap

The end goal is to have a markup language with features similar to LaTeX.

- [x] Checkboxes
- [x] Emojis (\:emoji:)
- [x] Colors
- [x] Watching and rendering on change
- [ ] Metadata files
- [x] Bibliography
- [ ] Math
- [ ] Text sizes
- [ ] Title pages
- [ ] Glossary
- [ ] Cross References
- [ ] Figures
- [ ] EPUB Rendering (PDF is too hard)
- [ ] Custom Elements via templates (50%)
- [ ] Custom Stylesheets


## License
[![FOSSA Status](https://app.fossa.com/api/projects/git%2Bgithub.com%2FTrivernis%2Fsnekdown.svg?type=large)](https://app.fossa.com/projects/git%2Bgithub.com%2FTrivernis%2Fsnekdown?ref=badge_large)