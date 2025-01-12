# Implementation of disabling interrupts

This module is used to provide atomic CAS for targets where atomic CAS is not available in the standard library.

- On MSP430 and AVR, they are always single-core, so this module is always used.
- On ARMv6-M (thumbv6m), pre-v6 ARM (e.g., thumbv4t, thumbv5te), RISC-V without A-extension, they could be multi-core, so this module is used when `--cfg portable_atomic_unsafe_assume_single_core` is enabled.

The implementation uses privileged instructions to disable interrupts, so it usually doesn't work on unprivileged mode.
Enabling this cfg in an environment where privileged instructions are not available, or if the instructions used are not sufficient to disable interrupts in the system, it is also usually considered **unsound**, although the details are system-dependent.

Consider using the [`critical-section` feature](../../../README.md#optional-features-critical-section) for systems that cannot use `--cfg portable_atomic_unsafe_assume_single_core`.

For some targets, the implementation can be changed by explicitly enabling cfg.

- On ARMv6-M, this disables interrupts by modifying the PRIMASK register.
- On pre-v6 ARM, this disables interrupts by modifying the I (IRQ mask) bit of the CPSR.
- On pre-v6 ARM with `--cfg portable_atomic_disable_fiq`, this disables interrupts by modifying the I (IRQ mask) bit and F (FIQ mask) bit of the CPSR.
- On RISC-V (without A-extension), this disables interrupts by modifying the MIE (Machine Interrupt Enable) bit of the `mstatus` register.
- On RISC-V (without A-extension) with `--cfg portable_atomic_s_mode`, this disables interrupts by modifying the SIE (Supervisor Interrupt Enable) bit of the `sstatus` register.
- On MSP430, this disables interrupts by modifying the GIE (Global Interrupt Enable) bit of the status register (SR).
- On AVR, this disables interrupts by modifying the I (Global Interrupt Enable) bit of the status register (SREG).

Some operations don't require disabling interrupts (loads and stores on targets except for AVR, but additionally on MSP430 `add`, `sub`, `and`, `or`, `xor`, `not`). However, when the `critical-section` feature is enabled, critical sections are taken for all atomic operations.

Feel free to submit an issue if your target is not supported yet.
