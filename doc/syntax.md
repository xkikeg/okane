# Accepted ledger syntax

okane accepts files in Ledger format. Unfortunately it's not obvious on the [ledger-cli 3.0 official document](https://www.ledger-cli.org/3.0/doc/ledger3.html)what is the exact syntax of Ledger format. This document explains the syntax that okane can handle.

## directives

Ledger format consists of a list of directives.

```ebnf
ledger-file ::= (directive vertical-space*)*

directive ::= transaction
            | top-comment
            | account-declaration
            | commodity-declaration
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

posting ::= posting-line metadata? new-line (metadata new-line)*

posting-line ::= sp+ account posting-value?

; account can't contain \t or two spaces
account ::= no-sp (no-sp | " " no-sp)*

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

metadata ::= ";" (metadata-key-value | metadata-tag-words | metadata-comment)

metadata-key-value ::= sp* tag sp* ":" sp* no-new-line*
                     | sp* tag sp* "::" sp* expr ; TODO(#78)

metadata-tag-words ::= sp* ":" (tag ":")+

metadata-comment ::= ";" no-new-line*

tag ::= <no-sp except ":">+
```

### Top level comments

In Ledger format, you can contain comments which is completely no-op and won't have any meanings.

```ebnf
top-level-comment ::= (comment-prefix no-new-line* new-line)+

comment-prefix ::= [;#%|*]
```

### account declaration

Ledger format allows you to declare the account. Using the declaration, an account can have descriptive note or aliases.

```ebnf
account-declaration ::= "account" sp+ account sp* new-line account-detail*

account-detail ::= account-note
                 | account-alias
                 | account-comment

; FYI information of the account.
; Currently it's no-op.
account-note ::= sp+ "note" sp+ no-new-line* new-line

; Declares alias of the account.
account-alias ::= sp+ "alias" sp+ account new-line

; Comment is pure no-op comment.
account-comment ::= sp+ comment-prefix no-new-line* new-line
```

### commodity declaration

Ledger format allows you to declare the commodity. Using the declaration, an commodity can have descriptive note or aliases.

```ebnf
commodity-declaration ::= "commodity" sp+ commodity sp* new-line commodity-detail*

commodity-detail ::= commodity-note
                   | commodity-alias
                   | commodity-format
                   | commodity-comment

; FYI information of the commodity.
; Currently it's no-op.
commodity-note ::= sp+ "note" sp+ no-new-line* new-line

; Declares alias of the commodity.
commodity-alias ::= sp+ "alias" sp+ commodity new-line

; Comment is pure no-op comment.
commodity-comment ::= sp+ comment-prefix no-new-line* new-line
```

### apply directives

## Expressions

Ledger allows to use expression in various places, including basic arithmetic operations.

```ebnf
value-expr ::= amount-expr | paren-expr

paren-expr ::= "(" sp* add-expr sp* ")"

add-expr ::= mul-expr (sp* [+-] sp* add-expr)*

mul-expr ::= unary-expr (sp* [*/] sp* unary-expr)*

unary-expr ::= "-"? value-expr

; Not supporting a prefix commodity like $100.
amount-expr ::= comma-decimal commodity?
```

## primitives

Here some primitive data structures are defined.

```ebnf
comma-decimal ::= comma-integer ("." decimal-number*)?

comma-integer ::= number+ | number{1-3} ("," number{3})*

number ::= [0-9]

commodity ::= [^- \t\r\n0123456789.,;:?!+*/^&|=<>[](){}@]

date ::= <yyyy-mm-dd> | <yyyy-mm-dd>
```

## characters

Here some basic character classes are defined.

```ebnf
vertical-space ::=  sp* new-line

sp ::= " " | "\t"

no-sp ::= ; any char except Unicode white space https://doc.rust-lang.org/std/primitive.char.html#method.is_whitespace

new-line ::= "\r"? "\n" | <EOF>

no-new-line ::= [^\r\n]
```