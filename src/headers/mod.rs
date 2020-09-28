pub(crate) mod exth;
pub(crate) mod header;
pub(crate) mod mobih;
pub(crate) mod palmdoch;

pub use self::{
    exth::ExtHeader,
    header::Header,
    mobih::{MobiHeader, TextEncoding},
    palmdoch::PalmDocHeader,
};

/// Trait allowing generic reading of header fields
pub(crate) trait HeaderField {
    /// Returns a position in the text where this field can be read
    fn position(self) -> u64;
}
