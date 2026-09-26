// The source-shared hypervisor module contains validation-image feature cfgs
// declared only by the hypervisor member. Host tests compile its pure tracker
// without enabling those target-only features.
#[allow(unexpected_cfgs)]
#[path = "../../../hypervisor/src/boot/lifecycle.rs"]
mod lifecycle;
