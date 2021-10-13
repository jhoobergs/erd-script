# Syntax

In Entity-Relation diagrams the following concepts exist:

- Entities
- Relations
- Attributes
- Identifying attributes
- (more exist, but are not yet supported)

## General info

- A `name` can only contain alphanumerical characters.
- A `label` can contain any character.

### Comments

You can add comments to the `erd` code by preceding it with `//`.

```erd
// This is a comment
```

## Entities

An entity is defined with the keyword `entity` followed by a `name`.

```erd
entity course
```

## Relations

A relation is defined with the keyword `relation` followed by a `name`.
This relation definition needs to be followed by at least 2 lines specifying the members.

### Members

The syntax for a member is of the form `<cardinality> <optionality> <entity_name>`

Valid values are:

- `cardinality`: `one` (maximum one), `multiple` (can be several), `exactly(n)` (exactly n times)
- `optionality`: `required` or `optional`

```erd
relation CI
  one required institute // A course has to be taught in an institute and can only be taught in one institute
  multiple optional course // An institute can teach multiple courses but does not need to teach one
```

It is possible to add a `label` to a relation by placing it between round brackets.

```erd
relation CI(C-I)
  one required institute
  multiple optional course
```

## Attributes

Attributes can be added to entities and relations.

The syntax for an attribute is of the form `<type> <name>` where type can be `id` or `attribute`.

```erd
entity course
  attribute name
  id code

entity institute
  attribute name

relation CI(C-I)
  one required institute
  multiple optional course
  attribute year
```
