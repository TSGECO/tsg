//! TSG is graph representation format that is designed to express Transcripts.
pub use tsg_core::*;

#[cfg(feature = "btsg")]
#[doc(inline)]
pub use btsg;
