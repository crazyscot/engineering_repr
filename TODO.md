I think we need our error type to impl StdError for qcp?
/ impl it, basics
/ consider implementing source i.e. pivot from enum to struct holding box/dyn/stderror. No, maybe one day if we get more advanced errors.

/Is there any change we can remove crates we don't need?
    traits - needed
    rational - will be needed soon
/    serde - hmm

dependabot !

Negative exponent. Should be pretty straightforward actually. But open a ticket...
- add to_ratio
- To String
- From String
- To Float Lossy [for all storage types...]
- grep "negative expo" straggler comment.
...
