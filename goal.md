# Near future

_Right now these are consumer-based wrappers, they don't represent runtime
implementation._

## Why?

---

### Actual differential testing
    1. For a rust runtime, I need to be able to test the runtime itself. This means
    a wrapper on top of the pjrt c api, which is rrad_pjrt or any other components chosen for rust plugin. 
    But the actual heavy lifting, concerns the actual replacement of the runtime.
    
    2. With this wrapper, having the infrastructure to move with direct plugin support
    is possible.

---

## What next?

### Wrapper Completion

    1. The goal is to keep building this wrapper level, and to begin structureing actual
    runtime implementations beginning with the pjrt componenet. What remains is mainly tests,
    and documentation, but the the focus now is runtime.


