# Alloy Parser

## Oddities

you can start an identifier using ( and end it with ) instead of "".
the advantage is, that this can be done recursively.

One oddity is that ((x)) will be parsed as the identifier "(x)".
This is particulary nice for inline css.
