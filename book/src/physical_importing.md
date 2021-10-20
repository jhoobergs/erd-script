# Importing the SQL in your dbms

## LibreOffice Base

- Start the program
- Create a new (or open a) database of type `HSQLDB Embedded`
- Click in the toolbar on `Tools` -> `SQL...`
- Paste the SQL code in the upper input field
  - Make sure you selected `LibreOffice Base` as dbms when generating the SQL code.
- Click on `Execute`
  - The status should be `1: Command successfully executed.`
- Close the SQL view (by clicking on Close in the right bottom corner)
- Click on `Tables` in the left menu
- Refresh the tables by clicking on `View` -> `Refresh Tables`
  - All tables should be there

You can also check the relations by clicking on `Tools` -> `Relationships`

## MS Access

- Start the program
- Create a new (or open a) database
- Click in the toolbar on `Create` -> `Query Design`
- Click in the right bottom corner on `SQL`
  - Or in the left upper corner
- Paste the SQL code here, command by command
  - MS Access does not support running multiple queries at once, so you will need to run each command (the things separated by ;) by hand
  - You execute the query by clicking on `Execute` in the left upper corner
- Close the query (without saving it)
- The tables should be created

You can also check the relations by clicking on `Database Tools` -> `Relationships`.
