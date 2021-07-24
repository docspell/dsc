pub const DOCSPELL_AUTH: &str = "X-Docspell-Auth";
pub const DOCSPELL_ADMIN: &str = "Docspell-Admin-Secret";

/// Extracts the filename from a Content-Disposition header
pub fn filename_from_header(header_value: &str) -> Option<&str> {
    header_value
        .find("filename=")
        .map(|index| &header_value[9 + index..])
        .map(|rest| rest.trim_matches('"'))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unit_filename_from_header() {
        assert_eq!(
            filename_from_header("inline; filename=\"test.jpg\""),
            Some("test.jpg")
        );
    }
}
