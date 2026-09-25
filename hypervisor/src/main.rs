//! Zelyr hypervisor image — P1 EL2 minimum bring-up.
//!
//! Contracts: the P1-W01 reference boot contract (pre-transfer tier,
//! rejection reporter, entry-state table) and the P1-W02 minimal runtime
//! design (establishment sequence, boot context, panic route, build
//! identity). The runtime establishes stages 1–8 of the W02 establishment
//! order and terminates through the recorded route at the unlinked W09
//! seam. The W03 inventory and W06 console mechanisms are linked but their
//! W09 phase callers are not; later packages own vectors, baseline and MMU.

#![no_std]
#![no_main]

mod arch;
mod boot;
