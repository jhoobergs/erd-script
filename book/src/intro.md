# Introduction

`erd-script` is a system to easily draw Entity-Relation (ER) diagrams.

In this case 'easily' means without needing to pay attention to the drawing part itself. This implies that the writer only needs to think about specifying the right entities, relations, attributes and identifiers.

It uses `graphviz` to actually draw to diagrams.

You can try it out in the browser as described [here](./online.md).

## Sample

```erd
// Modelled after https://graphviz.org/Gallery/undirected/ER.html

entity course
  attribute name
  id code

entity institute
  attribute name

entity student
  attribute name
  attribute number
  attribute grade

relation CI(C-I)
  one required institute
  multiple optional course

relation SI(S-I)
  one required institute
  multiple required student

relation SC(S-C)
  multiple optional course
  multiple optional student
```
