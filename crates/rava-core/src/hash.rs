use crate::protocol::SHA256_PREFIX;

pub fn is_sha256_hash(value: &str) -> bool {
    let Some(hex) = value.strip_prefix(SHA256_PREFIX) else {
        return false;
    };

    hex.len() == 64
        && hex
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}
