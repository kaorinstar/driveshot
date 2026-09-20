//! Which cloud drive a file was uploaded to.

use serde::{Deserialize, Serialize};
use std::fmt;

/// A cloud drive Driveshot can upload to.
///
/// All three are named here from the start, although they are not all implemented at once: the
/// identifier of a provider is written into every stored record, and a record written today has
/// to still be readable once the next provider is added. Adding a variant later would leave the
/// records already on disk naming a provider this enum does not know.
///
/// The serialized form is the value of [`Provider::id`], which is stable and is what the record
/// index on disk carries. Do not change those strings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[non_exhaustive]
pub enum Provider {
    /// Google Drive.
    GoogleDrive,
    /// Microsoft OneDrive.
    OneDrive,
    /// Dropbox.
    Dropbox,
}

impl Provider {
    /// Every provider, in the order the settings screen offers them.
    pub const ALL: [Provider; 3] = [Provider::GoogleDrive, Provider::OneDrive, Provider::Dropbox];

    /// The identifier used in stored records and in settings.
    ///
    /// This is the value that ends up on disk, so it is deliberately not the display name: a
    /// display name is translated, and a translation must never change what a file on disk says.
    #[must_use]
    pub const fn id(self) -> &'static str {
        match self {
            Provider::GoogleDrive => "google-drive",
            Provider::OneDrive => "one-drive",
            Provider::Dropbox => "dropbox",
        }
    }

    /// The name shown to the user, as the service itself writes it.
    #[must_use]
    pub const fn display_name(self) -> &'static str {
        match self {
            Provider::GoogleDrive => "Google Drive",
            Provider::OneDrive => "OneDrive",
            Provider::Dropbox => "Dropbox",
        }
    }

    /// The provider an identifier names, or `None` if no provider uses that identifier.
    ///
    /// A record written by a newer version can name a provider this version does not know, which
    /// is why this returns an option rather than failing.
    #[must_use]
    pub fn from_id(id: &str) -> Option<Provider> {
        Provider::ALL.into_iter().find(|p| p.id() == id)
    }
}

impl fmt::Display for Provider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.display_name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_identifier_maps_back_to_its_provider() {
        for provider in Provider::ALL {
            assert_eq!(Provider::from_id(provider.id()), Some(provider));
        }
    }

    #[test]
    fn identifiers_are_unique() {
        let mut ids: Vec<&str> = Provider::ALL.iter().map(|p| p.id()).collect();
        ids.sort_unstable();
        let count = ids.len();
        ids.dedup();
        assert_eq!(ids.len(), count, "two providers share an identifier");
    }

    #[test]
    fn an_unknown_identifier_is_not_a_provider() {
        assert_eq!(Provider::from_id("icloud"), None);
        assert_eq!(Provider::from_id(""), None);
        // The identifier is matched exactly, so a display name is not one.
        assert_eq!(Provider::from_id("Google Drive"), None);
    }

    // The identifiers below are written into the record index on disk. Changing one of them makes
    // every record already stored unreadable, so this test states them literally rather than
    // deriving them from the code it is checking.
    #[test]
    fn identifiers_are_the_ones_written_on_disk() {
        assert_eq!(Provider::GoogleDrive.id(), "google-drive");
        assert_eq!(Provider::OneDrive.id(), "one-drive");
        assert_eq!(Provider::Dropbox.id(), "dropbox");
    }

    #[test]
    fn the_serialized_form_is_the_identifier() {
        let json = serde_json::to_string(&Provider::GoogleDrive).expect("a provider serializes");
        assert_eq!(json, "\"google-drive\"");

        let provider: Provider =
            serde_json::from_str("\"dropbox\"").expect("a known identifier deserializes");
        assert_eq!(provider, Provider::Dropbox);
    }
}
