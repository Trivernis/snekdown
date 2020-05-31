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