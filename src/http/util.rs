pub const DOCSPELL_AUTH: &'static str = "X-Docspell-Auth";
pub const DOCSPELL_ADMIN: &'static str = "Docspell-Admin-Secret";

/// Extracts the filename from a Content-Disposition header
pub fn filename_from_header<'a>(header_value: &'a str) -> Option<&'a str> {
    header_value
        .find("filename=")
        .map(|index| &header_value[9 + index..])
        .map(|rest| rest.trim_matches('"'))
}
