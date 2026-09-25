//! Zelyr hypervisor image — P1 EL2 minimum bring-up.
//!
//! Contracts: the P1-W01 reference boot contract (pre-transfer tier,
//! rejection reporter, entry-state table) and the P1-W02 minimal runtime
//! design (establishment sequence, boot context, panic route, build
//! identity). W09 composes the W01–W08 mechanisms through Stage-1 into a
//! stable EL2 idle state; each phase retains its owning package's mechanism.

#![no_std]
#![no_main]

mod arch;
mod boot;
