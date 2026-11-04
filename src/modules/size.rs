pub fn size(bytes: u64) -> Option<String> {
    if bytes == 0 {
        return Some("0 B".into());
    }
    if bytes < 1024 {
        return Some(format!("{bytes} B"));
    }
    if bytes < 1024 * 1024 {
        return Some(format!("{:.2} KB", bytes as f64 / 1024.0));
    }
    if bytes < 1024 * 1024 * 1024 {
        return Some(format!("{:.2} MB", bytes as f64 / 1024.0 / 1024.0));
    }
    Some(format!("{:.2} GB", bytes as f64 / 1024.0 / 1024.0 / 1024.0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_the_largest_unit() {
        assert_eq!(size(1536).as_deref(), Some("1.50 KB"));
        assert_eq!(size(5 * 1024 * 1024 * 1024).as_deref(), Some("5.00 GB"));
    }
}
