# Yet another markdown standard

This projects goal is to implement a fast markdown parser with an extended syntax fitted
for my needs.

## Syntax

### Images

```
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

```
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

```
<[path]
```