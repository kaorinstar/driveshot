//! The error type this crate returns.

/// What can go wrong in this crate.
///
/// The variants are few on purpose. This crate does no I/O, so the failures it can report are
/// about the shape of the data it is handed, not about a file or a network.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// The stored index of uploads could not be read as JSON, or could not be written as JSON.
    #[error("the record index is not valid JSON: {0}")]
    Json(#[from] serde_json::Error),

    /// A record was added under an identifier the index already holds.
    ///
    /// Two records with the same identifier would make deletion ambiguous: the caller would have
    /// no way to say which of the two it meant. The caller generates these identifiers, so this
    /// is a bug on its side rather than something a user can cause.
    #[error("the index already holds a record with the identifier '{0}'")]
    DuplicateId(String),

    /// The stored index was written by a later version of Driveshot than this one.
    ///
    /// Reading it as if it were this version's shape would lose whatever that version added, so
    /// it is refused instead. The user is asked to update rather than shown a partial list.
    #[error("the record index was written by a later version of Driveshot (format {0})")]
    UnsupportedIndexVersion(u32),
}

/// The result type this crate returns.
pub type Result<T> = std::result::Result<T, Error>;
