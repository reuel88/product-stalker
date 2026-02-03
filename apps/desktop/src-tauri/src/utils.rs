use uuid::Uuid;

use crate::error::AppError;

/// Parse a string into a UUID, returning a validation error if invalid
pub fn parse_uuid(id: &str) -> Result<Uuid, AppError> {
    Uuid::parse_str(id).map_err(|_| AppError::Validation(format!("Invalid UUID: {}", id)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_uuid_valid() {
        let uuid_str = "550e8400-e29b-41d4-a716-446655440000";
        let result = parse_uuid(uuid_str);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().to_string(), uuid_str);
    }

    #[test]
    fn test_parse_uuid_invalid() {
        let result = parse_uuid("not-a-uuid");
        assert!(result.is_err());
        match result {
            Err(AppError::Validation(msg)) => {
                assert!(msg.contains("Invalid UUID"));
                assert!(msg.contains("not-a-uuid"));
            }
            _ => panic!("Expected Validation error"),
        }
    }

    #[test]
    fn test_parse_uuid_empty() {
        let result = parse_uuid("");
        assert!(result.is_err());
    }
}
