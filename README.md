# Yet another markdown standard

This projects goal is to implement a fast markdown parser with an extended syntax fitted
for my needs.

## Syntax

### URLs

```
The simplest way to insert an url is
(valid url)

The default syntax can also be used
[text](valid url)
```

### Images

```
Simple Syntax
!(valid url)

Extended syntax with a description
![description](valid url)

Extended syntax with metadata to specify the size
![description](valid url)[metadata]

Extended syntax with metadata and no description
!(valid url)[metadata]
```

### Quotes

```
Simple (default) Syntax
> This is a quote

Multiline
> This is a 
> Multiline Quote

Quote with metadata (e.g. Author)
[Trivernis - 2020]> This is a quote with metadata
```