## rrad_pjrt - rrad's internal pjrt binding

---
Goal: Contained pjrt binding of xla/pjrt(https://openxla.org/xla/pjrt), where the goal was to make a stable API for interacting
with the pjrt. This is a first of a long attempt to bring a full xla/pjrt stack to rust. I believe it's possible, and the first
step is to make a stable binding and slowly 'chip' away at the rest.

---
## Components

### Client
    This is the main entry point for interacting with the pjrt. A client can either be a single thread or multi-threaded.



