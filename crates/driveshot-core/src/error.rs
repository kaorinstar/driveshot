//! The error type this crate returns.

/// What can go wrong in this crate.
///
/// The variants are few on purpose. This crate does no I/O, so the failures it can report are
/// about the shape of the data it is handed, not about a file or a network. The three
/// authorization variants are of that kind too: they describe a redirect that arrived, not the
/// network it arrived over.
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

    /// The authorization server refused, which includes the user pressing Cancel.
    ///
    /// The string is the `error` code the provider sent, unchanged. `access_denied` is the
    /// ordinary one and is not a fault: it is what the consent screen's Cancel button produces.
    #[error("the drive did not grant access: {0}")]
    AuthorizationDenied(String),

    /// The redirect carried no `state`, or one that was not the one this sign-in sent out.
    ///
    /// The redirect arrives on a loopback port, which anything else on the machine can reach, so
    /// this is the check that says the redirect belongs to this sign-in. Whatever it carries is
    /// not exchanged.
    #[error("the sign-in reply did not come from the sign-in that was started")]
    AuthorizationStateMismatch,

    /// The redirect's state was right, but it carried neither an authorization code nor an error.
    #[error("the sign-in reply carried no authorization code")]
    AuthorizationIncomplete,
}

/// The result type this crate returns.
pub type Result<T> = std::result::Result<T, Error>;
