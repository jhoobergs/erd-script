# erd-script

`erd-script` is a system to easily draw Entity-Relation (ER) diagrams.

In this case 'easily' means without needing to pay attention to the drawing part itself. This implies that the writer only needs to think about specifying the right entities, relations, attributes and identifiers.

It uses graphviz to actually draw to diagrams.

# Usage

The web interface is available at [github.io](https://jhoobergs.github.io/erd-script/).

The cli isn't created yet.

# Features
Currently following 'parts' of ER-diagrams are supported:
  - `entities` with `attributes` and `identifiers`
  - `relations`
    - Members:
      - `entity`
      - Cardinality (`one`, `exactly`, `multiple`)
      - Optionality (`optional` or `required`)
    - `attributes` and `identifiers`  

Parts that are not supported yet:
  - Weak entities
  - Composed attributes
  - ...

# Folders
- `book`: Contains (a first version of) a book describing the usage of `erd-script`
- `erd-wasm`: The wasm module used to create the web interface of `erd-script`
- `erd`: The main rust crate containing `erd-script`
- `examples`: Some examples of `erd-script`
