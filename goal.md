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


## Runtime 
    The main goal now is rust based pjrt runtime compatability for xla. I want to validate this work against real JAX,
    so I still need parts of XLA in the loop. I plan to keep two comparable setups: one using standard XLA PJRT, and one
    using a Rust PJRT runtime. rrad_pjrt helps with API shaping, parity checks, and benchmarking support, but it is not 
    the runtime implementation itself. The runtime target is a cdylib (rrad_pjrt_runtime) that exports the full required 
    PJRT extern function table. Once complete, that Rust runtime should plug into a working XLA/JAX environment as the 
    PJRT provider, with any required extension wiring adjusted for the Rust backend.


### Wrapper Completion

    1. The goal is to keep building this wrapper level, and to begin structureing actual
    runtime implementations beginning with the pjrt componenet. What remains is mainly tests,
    and documentation, but the the focus now is runtime.


