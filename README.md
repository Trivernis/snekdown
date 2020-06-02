# Snekdown - A wonderful markdown parser

This projects goal is to implement a fast markdown parser with an extended syntax fitted
for my needs.

## Usage

```
USAGE:
    snekdown [OPTIONS] <input> <output>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -f, --format <format>     [default: html]

ARGS:
    <input>     
    <output>
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


### Quotes

```md
Simple (default) Syntax
> This is a quote

Multiline
> This is a 
> Multiline Quote

Quote with metadata (e.g. Author)
[Trivernis - 2020]> This is a quote with metadata
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
```

## Roadmap

The end goal is to have a markdown language similar to LaTeX.

- [ ] Checkboxes
- [ ] Emojis (\:emoji:)
- [ ] Bibliography
- [ ] Math
- [ ] Figures
- [ ] Text sizes
- [ ] Colors
- [ ] Cross References
- [ ] Title pages
- [ ] Glossary
- [ ] EPUB Rendering (PDF is too hard
- [ ] Custom Elements via templates
