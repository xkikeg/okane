# Accepted ledger syntax

okane accepts files in ledger format. However, reading [ledger-cli 3.0 official document](https://www.ledger-cli.org/3.0/doc/ledger3.html), it's not obvious what is the strictly exact syntax. This document explains what syntax okane can parse.

## directives

Ledger format consists of a list of directives.

```ebnf
ledger-file ::= (directive vertical-space*)*

directive ::= transaction
            | top-comment
            | apply-tag
            | end-apply-tag
            ; TODO: more directives must be supported
```

### Transaction

Transaction is the most important directive.

```ebnf
transaction ::= transaction-header metadata* posting*

transaction-header ::= transaction-date (sp+ transcation-note)? (new-line | metadata)

; secondary date is called "effective date",
; which can be used to represent the date when the transaction took effect after some delay.
transaction-date ::= date ("=" date)?

transcation-note ::= (clear-state sp*)? (transaction-code sp*)? payee

clear-state ::= "*" | "!"

transaction-code ::= "(" sp* [^()\r\n]* sp* ")"

payee ::= [^\r\n;]*

posting ::= posting-line (new-line | metadata) metadata*

posting-line ::= sp+ posting-account posting-value?

; posting-account can't contain \t or two spaces
posting-account ::= (no-sp | " " no-sp)+

posting-value ::= ("  " | "\t") sp* (posting-amount sp*)? balance?

posting-amount ::= value-expr sp* posting-lot? posting-cost?

posting-lot ::= (lot-price sp*)? (lot-date sp*)? (lot-note sp*)?
              | (lot-price sp*)? (lot-note sp*)? (lot-date sp*)?
              | ... ; permutations

lot-price ::= "{{" sp* amount-expr sp* "}}" ; total
            | "{"  sp* amount-expr sp* "}"  ; rate

lot-date ::= "[" sp* date sp* "]"

lot-note ::= "(" [^()@]* ")"

posting-cost ::= "@@" sp* value-expr  ; total
               | "@"  sp* value-expr  ; rate

balance ::= "=" sp* value-expr sp*

metadata ::= ";" (key-value | tag-words | comment) new-line

key-value ::= sp* tag sp* ":" sp* no-new-line*

tag-words ::= sp* ":" (tag ":")+

comment ::= ";" no-new-line*

tag ::= no-sp+
```

## Top level comments

Ledger file can contain comments. It's similar to transaction metadata,
but it won't have any meaning for transaction data.

```ebnf
top-level-comment ::= ([;#%|*] no-new-line* new-line)+
```

## Expressions

TODO

## primitives

Here some primitive data structures are defined.

```ebnf
```

## characters

Here some basic character classes are defined.

```ebnf
vertical-space ::=  sp* new-line

sp ::= " " | "\t"

no-sp ::= ; any char except Unicode white space https://doc.rust-lang.org/std/primitive.char.html#method.is_whitespace

new-line ::= "\r"? "\n"

no-new-line ::= [^\r\n]
```