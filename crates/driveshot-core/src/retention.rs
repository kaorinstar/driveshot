//! How long an uploaded file is kept before it is deleted again.

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::num::NonZeroU32;

/// How long a file stays on the cloud drive after it was uploaded.
///
/// The shortest retention that can be expressed is one day. Zero is not a retention period but an
/// instruction to delete what was just uploaded, and an accidental zero - a cleared input field, a
/// setting that was never filled in - would delete a file the user had just shared. The type
/// refuses to hold it, so that case cannot arise from a settings file either.
///
/// The serialized form is tagged, so a value reads plainly in the stored index:
///
/// ```json
/// {"kind": "forever"}
/// {"kind": "days", "days": 7}
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum Retention {
    /// The file is kept until the user deletes it themselves. Driveshot never deletes it.
    Forever,
    /// The file is deleted once this many days have passed since it was uploaded.
    Days {
        /// The number of days, which is at least one.
        days: NonZeroU32,
    },
}

impl Retention {
    /// A retention of `days` days, or `None` if `days` is zero.
    ///
    /// Use this for a number that came from a settings file or from an input field, where zero is
    /// a mistake rather than an instruction.
    #[must_use]
    pub fn days(days: u32) -> Option<Retention> {
        NonZeroU32::new(days).map(|days| Retention::Days { days })
    }

    /// Whether Driveshot will ever delete a file kept under this retention.
    #[must_use]
    pub const fn is_forever(self) -> bool {
        matches!(self, Retention::Forever)
    }

    /// The moment a file uploaded at `uploaded_at` falls due for deletion.
    ///
    /// `None` means nothing is ever due: either the retention is [`Retention::Forever`], or the
    /// number of days is so large that the moment cannot be represented at all. Both are treated
    /// the same way on purpose. Failing to represent a date that far out must leave the file in
    /// place; the alternative - treating an unrepresentable date as one already past - would
    /// delete a file the user asked to keep.
    #[must_use]
    pub fn expires_at(self, uploaded_at: DateTime<Utc>) -> Option<DateTime<Utc>> {
        match self {
            Retention::Forever => None,
            Retention::Days { days } => {
                let span = Duration::try_days(i64::from(days.get()))?;
                uploaded_at.checked_add_signed(span)
            }
        }
    }

    /// Whether a file uploaded at `uploaded_at` is due for deletion at `now`.
    ///
    /// A file is due the moment its retention runs out, so a seven-day retention on a file
    /// uploaded at 09:00 is due at 09:00 seven days later, not a moment after.
    #[must_use]
    pub fn is_expired(self, uploaded_at: DateTime<Utc>, now: DateTime<Utc>) -> bool {
        self.expires_at(uploaded_at).is_some_and(|due| now >= due)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(text: &str) -> DateTime<Utc> {
        text.parse().expect("the test wrote a valid timestamp")
    }

    fn days(n: u32) -> Retention {
        Retention::days(n).expect("the test asked for at least one day")
    }

    #[test]
    fn zero_days_is_not_a_retention() {
        assert_eq!(Retention::days(0), None);
        assert!(Retention::days(1).is_some());
    }

    #[test]
    fn a_file_kept_forever_is_never_due() {
        let uploaded = at("2026-09-20T09:00:00Z");
        assert_eq!(Retention::Forever.expires_at(uploaded), None);
        assert!(!Retention::Forever.is_expired(uploaded, at("2099-01-01T00:00:00Z")));
        assert!(Retention::Forever.is_forever());
    }

    #[test]
    fn the_due_date_is_the_upload_plus_the_days() {
        let uploaded = at("2026-09-20T09:00:00Z");
        assert_eq!(
            days(7).expires_at(uploaded),
            Some(at("2026-09-27T09:00:00Z"))
        );
    }

    // The boundary is the whole point of this type, so all three sides of it are stated.
    #[test]
    fn a_file_is_due_at_the_moment_its_retention_runs_out() {
        let uploaded = at("2026-09-20T09:00:00Z");
        let retention = days(7);

        assert!(!retention.is_expired(uploaded, at("2026-09-27T08:59:59Z")));
        assert!(retention.is_expired(uploaded, at("2026-09-27T09:00:00Z")));
        assert!(retention.is_expired(uploaded, at("2026-09-27T09:00:01Z")));
    }

    #[test]
    fn a_clock_that_went_backwards_does_not_delete_anything() {
        let uploaded = at("2026-09-20T09:00:00Z");
        assert!(!days(1).is_expired(uploaded, at("2026-09-19T09:00:00Z")));
    }

    // A due date too far out to represent must leave the file alone rather than delete it.
    #[test]
    fn a_retention_beyond_the_calendar_keeps_the_file() {
        let uploaded = at("2026-09-20T09:00:00Z");
        let retention = days(u32::MAX);

        assert_eq!(retention.expires_at(uploaded), None);
        assert!(!retention.is_expired(uploaded, at("2099-01-01T00:00:00Z")));
    }

    #[test]
    fn the_stored_form_reads_plainly() {
        let json = serde_json::to_string(&days(7)).expect("a retention serializes");
        assert_eq!(json, r#"{"kind":"days","days":7}"#);

        let json = serde_json::to_string(&Retention::Forever).expect("a retention serializes");
        assert_eq!(json, r#"{"kind":"forever"}"#);

        let read: Retention =
            serde_json::from_str(r#"{"kind":"days","days":30}"#).expect("a retention reads back");
        assert_eq!(read, days(30));
    }

    // A settings file holding zero days is refused by the type rather than accepted as "delete at
    // once". Nothing in the application has to check for it separately.
    #[test]
    fn a_stored_zero_is_refused() {
        let read: std::result::Result<Retention, _> =
            serde_json::from_str(r#"{"kind":"days","days":0}"#);
        assert!(read.is_err(), "zero days was accepted from the stored form");
    }
}
