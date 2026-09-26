// The source-shared hypervisor module contains validation-image feature cfgs
// declared only by the hypervisor member. Host tests compile its pure tracker
// without enabling those target-only features.
#[expect(unexpected_cfgs, reason = "source-shared target-only W11 feature cfgs")]
#[path = "../../../hypervisor/src/boot/lifecycle.rs"]
mod lifecycle;
