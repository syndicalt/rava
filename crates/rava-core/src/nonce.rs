use uuid::Uuid;

pub fn is_canonical_uuid_v4(value: &str) -> bool {
    let Ok(uuid) = Uuid::parse_str(value) else {
        return false;
    };

    uuid.get_version_num() == 4 && uuid.to_string() == value
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_canonical_uuid_v4() {
        assert!(is_canonical_uuid_v4("550e8400-e29b-41d4-a716-446655440000"));
    }

    #[test]
    fn rejects_non_uuid_or_non_canonical_uuid() {
        assert!(!is_canonical_uuid_v4("not-a-uuid"));
        assert!(!is_canonical_uuid_v4(
            "550E8400-E29B-41D4-A716-446655440000"
        ));
        assert!(!is_canonical_uuid_v4(
            "550e8400-e29b-11d4-a716-446655440000"
        ));
    }
}
