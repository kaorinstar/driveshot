//! What was uploaded, where it went, and when.
//!
//! Deleting a file once its retention has run out needs a record of the upload: the cloud drive
//! knows when a file was created, but asking every provider about every file would mean a network
//! call before a decision can be made at all. Driveshot keeps its own index instead, and this
//! module is the shape of it.

use crate::{Error, Provider, Result, Retention};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// The number written into a stored index by this version.
///
/// It exists so that a future change of shape can be recognised rather than guessed at. An index
/// written by a newer version is refused with [`Error::UnsupportedIndexVersion`] instead of being
/// read as if it were this one.
pub(crate) const INDEX_VERSION: u32 = 1;

/// One upload: the file, the drive it went to, and the terms it is kept under.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShotRecord {
    /// Driveshot's own identifier for this record, unique within the index.
    ///
    /// The caller generates it. It is not the provider's identifier, because a record has to be
    /// nameable before the upload it describes has finished.
    pub id: String,

    /// The cloud drive the file was uploaded to.
    pub provider: Provider,

    /// The identifier the provider gave the file, used to delete it later.
    pub remote_id: String,

    /// The name the file carries on the drive.
    pub file_name: String,

    /// When the upload finished. The retention is counted from this moment.
    pub uploaded_at: DateTime<Utc>,

    /// The share link, once one has been published.
    ///
    /// `None` means the upload finished but no link was published, which is what a failure
    /// between the two steps leaves behind.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub share_url: Option<String>,

    /// How long this file is kept.
    ///
    /// It is stored per record rather than read from the settings when deletion is considered, so
    /// that changing the setting does not change the terms of a file already shared.
    pub retention: Retention,
}

impl ShotRecord {
    /// The moment this file falls due for deletion, or `None` if it is never due.
    #[must_use]
    pub fn expires_at(&self) -> Option<DateTime<Utc>> {
        self.retention.expires_at(self.uploaded_at)
    }

    /// Whether this file is due for deletion at `now`.
    #[must_use]
    pub fn is_expired(&self, now: DateTime<Utc>) -> bool {
        self.retention.is_expired(self.uploaded_at, now)
    }
}

/// Every upload Driveshot has made and has not yet deleted.
///
/// The index is the application's own list. A file deleted at the provider is removed from here as
/// well; a file this index does not name is not Driveshot's to delete.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShotIndex {
    records: Vec<ShotRecord>,
}

impl ShotIndex {
    /// An index holding nothing.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Every record, in the order it was added.
    #[must_use]
    pub fn records(&self) -> &[ShotRecord] {
        &self.records
    }

    /// How many records the index holds.
    #[must_use]
    pub fn len(&self) -> usize {
        self.records.len()
    }

    /// Whether the index holds nothing.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }

    /// The record with this identifier, if the index holds one.
    #[must_use]
    pub fn get(&self, id: &str) -> Option<&ShotRecord> {
        self.records.iter().find(|record| record.id == id)
    }

    /// Adds a record.
    ///
    /// # Errors
    ///
    /// [`Error::DuplicateId`] if the index already holds a record with the same identifier.
    /// Allowing two would make deletion ambiguous, and the caller generates these identifiers, so
    /// a collision is a bug rather than something a user can cause.
    pub fn insert(&mut self, record: ShotRecord) -> Result<()> {
        if self.get(&record.id).is_some() {
            return Err(Error::DuplicateId(record.id));
        }
        self.records.push(record);
        Ok(())
    }

    /// Removes the record with this identifier and returns it, or returns `None` if the index does
    /// not hold one.
    pub fn remove(&mut self, id: &str) -> Option<ShotRecord> {
        let position = self.records.iter().position(|record| record.id == id)?;
        Some(self.records.remove(position))
    }

    /// Every record due for deletion at `now`, oldest upload first.
    ///
    /// Deleting is the caller's job: a record stays in the index until the provider confirms the
    /// file is gone. A deletion that failed is therefore retried on the next pass rather than
    /// forgotten, which is what removing the record here would cause.
    #[must_use]
    pub fn due_for_deletion(&self, now: DateTime<Utc>) -> Vec<&ShotRecord> {
        let mut due: Vec<&ShotRecord> = self
            .records
            .iter()
            .filter(|record| record.is_expired(now))
            .collect();
        due.sort_by_key(|record| record.uploaded_at);
        due
    }

    /// Reads an index from the JSON form written by [`ShotIndex::to_json`].
    ///
    /// # Errors
    ///
    /// [`Error::Json`] if the text is not the JSON this expects, and
    /// [`Error::UnsupportedIndexVersion`] if it was written by a later version of Driveshot.
    pub fn from_json(text: &str) -> Result<Self> {
        let stored: StoredIndex = serde_json::from_str(text)?;
        if stored.version > INDEX_VERSION {
            return Err(Error::UnsupportedIndexVersion(stored.version));
        }
        Ok(ShotIndex {
            records: stored.records,
        })
    }

    /// Writes the index as JSON, indented, because a person reading the file is the point of a
    /// plain text format.
    ///
    /// # Errors
    ///
    /// [`Error::Json`] if a record cannot be written as JSON.
    pub fn to_json(&self) -> Result<String> {
        let stored = StoredIndex {
            version: INDEX_VERSION,
            records: self.records.clone(),
        };
        Ok(serde_json::to_string_pretty(&stored)?)
    }
}

/// The shape of the file on disk: the records, behind a version number.
///
/// The number is what lets a later change of shape be recognised. Without it, a file written by a
/// newer Driveshot would be read as if it were this one.
#[derive(Serialize, Deserialize)]
struct StoredIndex {
    version: u32,
    #[serde(default)]
    records: Vec<ShotRecord>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(text: &str) -> DateTime<Utc> {
        text.parse().expect("the test wrote a valid timestamp")
    }

    fn record(id: &str, uploaded_at: &str, retention: Retention) -> ShotRecord {
        ShotRecord {
            id: id.to_owned(),
            provider: Provider::GoogleDrive,
            remote_id: format!("remote-{id}"),
            file_name: format!("{id}.png"),
            uploaded_at: at(uploaded_at),
            share_url: Some(format!("https://example.invalid/{id}")),
            retention,
        }
    }

    fn days(n: u32) -> Retention {
        Retention::days(n).expect("the test asked for at least one day")
    }

    #[test]
    fn a_new_index_holds_nothing() {
        let index = ShotIndex::new();
        assert!(index.is_empty());
        assert_eq!(index.len(), 0);
        assert!(index
            .due_for_deletion(at("2099-01-01T00:00:00Z"))
            .is_empty());
    }

    #[test]
    fn a_record_can_be_added_found_and_removed() {
        let mut index = ShotIndex::new();
        index
            .insert(record("a", "2026-09-20T09:00:00Z", days(7)))
            .expect("the index was empty");

        assert_eq!(index.len(), 1);
        assert_eq!(index.get("a").map(|r| r.file_name.as_str()), Some("a.png"));

        let removed = index.remove("a").expect("the record was there");
        assert_eq!(removed.id, "a");
        assert!(index.is_empty());
        assert_eq!(index.remove("a"), None);
    }

    #[test]
    fn the_same_identifier_cannot_be_added_twice() {
        let mut index = ShotIndex::new();
        index
            .insert(record("a", "2026-09-20T09:00:00Z", days(7)))
            .expect("the index was empty");

        let second = index.insert(record("a", "2026-09-21T09:00:00Z", days(7)));
        assert!(matches!(second, Err(Error::DuplicateId(id)) if id == "a"));
        assert_eq!(index.len(), 1, "the refused record was stored anyway");
    }

    #[test]
    fn only_the_records_past_their_retention_are_due() {
        let mut index = ShotIndex::new();
        index
            .insert(record("old", "2026-09-01T09:00:00Z", days(7)))
            .expect("the index was empty");
        index
            .insert(record("fresh", "2026-09-20T09:00:00Z", days(7)))
            .expect("the identifier is new");
        index
            .insert(record("kept", "2026-01-01T09:00:00Z", Retention::Forever))
            .expect("the identifier is new");

        let due = index.due_for_deletion(at("2026-09-21T09:00:00Z"));
        let ids: Vec<&str> = due.iter().map(|record| record.id.as_str()).collect();
        assert_eq!(ids, ["old"]);
    }

    #[test]
    fn the_oldest_upload_is_due_first() {
        let mut index = ShotIndex::new();
        index
            .insert(record("second", "2026-09-02T09:00:00Z", days(1)))
            .expect("the index was empty");
        index
            .insert(record("first", "2026-09-01T09:00:00Z", days(1)))
            .expect("the identifier is new");

        let due = index.due_for_deletion(at("2026-09-20T09:00:00Z"));
        let ids: Vec<&str> = due.iter().map(|record| record.id.as_str()).collect();
        assert_eq!(ids, ["first", "second"]);
    }

    // A record stays until the provider confirms the file is gone, so that a failed deletion is
    // retried rather than forgotten.
    #[test]
    fn listing_what_is_due_does_not_remove_anything() {
        let mut index = ShotIndex::new();
        index
            .insert(record("old", "2026-09-01T09:00:00Z", days(1)))
            .expect("the index was empty");

        let now = at("2026-09-20T09:00:00Z");
        assert_eq!(index.due_for_deletion(now).len(), 1);
        assert_eq!(index.due_for_deletion(now).len(), 1);
        assert_eq!(index.len(), 1);
    }

    #[test]
    fn an_index_survives_a_round_trip_through_json() {
        let mut index = ShotIndex::new();
        index
            .insert(record("a", "2026-09-20T09:00:00Z", days(7)))
            .expect("the index was empty");
        index
            .insert(record("b", "2026-09-21T09:00:00Z", Retention::Forever))
            .expect("the identifier is new");

        let json = index.to_json().expect("the index writes as JSON");
        let read = ShotIndex::from_json(&json).expect("what was written reads back");
        assert_eq!(read, index);
    }

    #[test]
    fn a_record_without_a_share_link_reads_back() {
        let json = r#"{
            "version": 1,
            "records": [
                {
                    "id": "a",
                    "provider": "one-drive",
                    "remote_id": "remote-a",
                    "file_name": "a.png",
                    "uploaded_at": "2026-09-20T09:00:00Z",
                    "retention": {"kind": "forever"}
                }
            ]
        }"#;

        let index = ShotIndex::from_json(json).expect("the index reads");
        let record = index.get("a").expect("the record is there");
        assert_eq!(record.share_url, None);
        assert_eq!(record.provider, Provider::OneDrive);
    }

    #[test]
    fn an_index_from_a_later_version_is_refused_rather_than_guessed_at() {
        let json = r#"{"version": 99, "records": []}"#;
        let read = ShotIndex::from_json(json);
        assert!(matches!(read, Err(Error::UnsupportedIndexVersion(99))));
    }

    #[test]
    fn text_that_is_not_the_expected_json_is_an_error() {
        assert!(matches!(
            ShotIndex::from_json("not json"),
            Err(Error::Json(_))
        ));
        assert!(matches!(ShotIndex::from_json("{}"), Err(Error::Json(_))));
    }

    #[test]
    fn the_written_form_carries_its_version() {
        let json = ShotIndex::new().to_json().expect("an empty index writes");
        assert!(
            json.contains("\"version\": 1"),
            "the stored form has to name its version: {json}"
        );
    }
}
