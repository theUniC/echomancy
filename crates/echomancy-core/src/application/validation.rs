//! Shared validation helpers for the application layer.

use uuid::Uuid;

/// Returns `Ok(())` if `id` parses as a valid UUID.
///
/// On failure, calls `make_err(id)` to construct the error variant.
///
/// # Examples
///
/// ```ignore
/// validate_uuid("not-a-uuid", |id| ApplicationError::InvalidGameId { id: id.to_owned() })
///     .unwrap_err(); // returns the error
/// ```
pub(crate) fn validate_uuid<E>(id: &str, make_err: impl Fn(&str) -> E) -> Result<(), E> {
    if Uuid::parse_str(id).is_ok() {
        Ok(())
    } else {
        Err(make_err(id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_uuid_returns_ok() {
        let id = uuid::Uuid::new_v4().to_string();
        assert!(validate_uuid::<String>(&id, |s| s.to_owned()).is_ok());
    }

    #[test]
    fn invalid_uuid_returns_err_with_id() {
        let result = validate_uuid("not-a-uuid", |s: &str| s.to_owned());
        assert_eq!(result.unwrap_err(), "not-a-uuid");
    }

    #[test]
    fn empty_string_returns_err() {
        let result = validate_uuid("", |s: &str| s.to_owned());
        assert!(result.is_err());
    }
}
