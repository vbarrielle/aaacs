# aaacs

aaacs is a simple tool to balance spendings between multiple people. It is based
on a simple but very flexible representation: each transaction is paid for by
one of the users, and each user has a number of shares for each transaction,
representing how much he benefited from the transaction. From this
representation, aaacs will compute which user is a creditor and which is a
debitor, and by which amount.

For now the tool is CLI only, but expect a GUI in the future.

## Example

Suppose we have three friends, Eska, Simon and Shuba, who prepared a meal
together. Simon brought some wine, and Eska brought a tartiflette. However, Eska
did not drink any wine, while Shuba drank twice as much as Simon. And Shuba ate
twice as much as Simon, and Eska three times as much as Simon. These friends
want to divide the costs fairly. The tartiflette cost 42€, and the wine 15€.

Using aacs, they first modelize their meal with the following `input.yml` file:

```yml
---
users:
    - Simon
    - Shuba
    - Eska
purchases:
    - descr: wine
      amount: 15
      who: Simon
      benef_to_shares:
          Simon: 1
          Shuba: 2
    - descr: tartiflette
      amount: 42
      who: Eska
      benef_to_shares:
          Simon: 1
          Shuba: 2
          Eska: 3

```

And aacs procudes the following output:

```
$ cargo run input.yml
Processing accounts for input.yml:
Eska has a balance of: 22.91
Shuba has a balance of: -25.27
Simon has a balance of: 2.36
```

Now our friends know that Shuba ows 2.36€ to Simon, and 22.91 to Eska.

## Status and future work

Currently works with basic validation of the input yaml file. Different input
types will be considered in the future, for example tabular input in the form
of `.tsv`, `.csv` or `.ods` files would be nice.

I also plan to experiment with the various Rust GUI frameworks to build a simple
GUI.
