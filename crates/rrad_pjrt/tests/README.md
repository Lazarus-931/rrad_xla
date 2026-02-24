# PJRT Functionality Tests


These tests are meant to cover end-to-end functionality of the rrad_pjrt crate. Within it there are tests for the PJRT
loader and CPU runtime, testing, for example that the PJRT loader can load a PJRT module and execute it. It is broken up
into binding-specific files, where a centralized test helper is used to load runtime and client for each test. 100% coverage
is not guaranteed so far, around 70% covered.

A unified end-to-end test is available in `tests/unified.r`.

