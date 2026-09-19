//! Build identity per the P0-W16 version/build metadata contract.
//!
//! Sourcing, as recorded in the W02 implementation record: `project_version`
//! is the single project-version declaration (version-build-metadata §3.3);
//! `target_architecture` is the delivered build-target baseline's value
//! family. `build_profile`, `source_revision`, and `dirty` require the
//! P0-W16 embedding mechanism, which P0-W16 v0.1 deliberately does not
//! define; they degrade to the recorded unavailable literal per the W02
//! design's failure boundary — identity is never fabricated.

/// Degraded-field literal named by the W02 design's identity contract.
const UNAVAILABLE: &str = "unavailable";

/// The identity minimum set of the P0-W16 contract (§5.1 diagnostics
/// association); a plain immutable static, no allocation.
pub(crate) struct BuildIdentity {
    pub(crate) project_version: &'static str,
    pub(crate) target_architecture: &'static str,
    pub(crate) build_profile: &'static str,
    pub(crate) source_revision: &'static str,
    pub(crate) dirty: &'static str,
}

pub(crate) static BUILD_IDENTITY: BuildIdentity = BuildIdentity {
    project_version: "0.1.0",
    target_architecture: "aarch64",
    build_profile: UNAVAILABLE,
    source_revision: UNAVAILABLE,
    dirty: UNAVAILABLE,
};

/// Readiness assertion (establishment stage 8): the identity resolves —
/// every contracted field carries a declared value or the recorded
/// degradation literal, never an empty fabrication.
pub(crate) fn identity_resolvable() -> bool {
    let identity = &BUILD_IDENTITY;
    !identity.project_version.is_empty()
        && !identity.target_architecture.is_empty()
        && !identity.build_profile.is_empty()
        && !identity.source_revision.is_empty()
        && !identity.dirty.is_empty()
}
