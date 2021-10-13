# Syntax

`erd-script` supports converting an ER-diagram to physical structure by generating the `sql` code to create the tables and constraints.

To use this feature, extra syntax is needed.

The following concepts are used:

- Tables
- Datatypes

## Datatypes

Columns in a table need to have a datatype, therefore you need to add a type to each attribute in the ER-diagram.

```erd
entity Person
  id id type autoincrement
  attribute name type varchar(100)
  attribute alive type boolean
```

The following datatypes are supported

- `autoincrement`: an integer that is filled in automatically
- `boolean`: a boolean `true` / `false` value
- `varchar(n)`: a text of maximum length `n`
- `integer`: an integer
- `float`: a floating comma value
- `datetime`: a date and time

## Tables

### Entities

For each entitiy a table needs to be created with a command of the form `table <table_name> from entity <entity_name>`.

```erd
table person from entity Person
```

### Relations

#### Foreign keys

Relations of type `1:1` or `1:n` can be implemented by adding a foreign key to the right table.

```erd
table person from entity Person
foreign key father for Father
foreign key mother for Mother
```

#### Extra table

Relations of type `m:n` need to be implemented by creating an extra table with a command of the form `table <table_name> from relation <relation_name>`.

```erd
table ownership from relation Ownership
```

## Example

```erd
entity Person
  id id type autoincrement
  attribute name type varchar(100)
  attribute alive type boolean

relation Father
  one optional Person // For now, this order is important, the one with 'one' should be the first for unary relations
  multiple optional Person

relation Mother
  one optional Person
  multiple optional Person

relation Friends(Is friends with)
  multiple optional Person
  multiple optional Person

entity Car
  id id type autoincrement
  attribute color type varchar(20)
  attribute price type float

relation Ownership(Is owner)
  multiple optional Person
  multiple optional Car
  attribute since type datetime

table person from entity Person
foreign key father for Father
foreign key mother for Mother

table car from entity Car

table ownership from relation Ownership

table friendship from relation Friends
```
