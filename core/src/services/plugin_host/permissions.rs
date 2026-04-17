/// Check if the given permission set includes the required permission.
/// "write" implicitly includes "read".
pub fn has_permission(permissions: &[String], required: &str) -> bool {
    permissions
        .iter()
        .any(|p| p == required || (p == "write" && required == "read"))
}

/// Check if the given permission set satisfies a permission tier.
///   Tier 1: read access (read or write)
///   Tier 2: write access
///   Tier 3: system access (fs or net)
#[allow(dead_code)]
pub fn check_tier(permissions: &[String], tier: u8) -> bool {
    match tier {
        1 => has_permission(permissions, "read"),
        2 => has_permission(permissions, "write"),
        3 => has_permission(permissions, "fs") || has_permission(permissions, "net"),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_has_read_permission() {
        let perms = vec!["read".to_string()];
        assert!(has_permission(&perms, "read"));
        assert!(!has_permission(&perms, "write"));
    }

    #[test]
    fn test_write_implies_read() {
        let perms = vec!["write".to_string()];
        assert!(has_permission(&perms, "read"));
        assert!(has_permission(&perms, "write"));
    }

    #[test]
    fn test_check_tier() {
        let read = vec!["read".to_string()];
        assert!(check_tier(&read, 1));
        assert!(!check_tier(&read, 2));

        let write = vec!["write".to_string()];
        assert!(check_tier(&write, 1));
        assert!(check_tier(&write, 2));

        let fs = vec!["fs".to_string()];
        assert!(check_tier(&fs, 3));
        assert!(!check_tier(&fs, 1));
    }
}
