pub const DOCSPELL_AUTH: &str = "X-Docspell-Auth";
pub const DOCSPELL_ADMIN: &str = "Docspell-Admin-Secret";

use percent_encoding::percent_decode;

// Couldn't find a library for parsing the header properly ¯\_(ツ)_/¯

/// Extracts the filename from a Content-Disposition header It prefers
/// 'filename*' values over 'filename' should both be present.
pub fn filename_from_header(header_value: &str) -> Option<String> {
    log::debug!("file header value: {}", header_value);
    let mut all: Vec<(u32, String)> = header_value
        .split(';')
        .map(|e| e.trim())
        .filter_map(decode_name)
        .collect();

    all.sort_by(|(a, _), (b, _)| a.partial_cmp(b).unwrap());

    let name = all.into_iter().next().map(|(_, e)| e);

    log::debug!("Return file name: {:?}", name);
    name
}

fn decode_name(v: &str) -> Option<(u32, String)> {
    from_percent_encoded(v).or_else(|| from_basic_name(v).map(|(n, s)| (n, s.to_string())))
}

fn from_basic_name(v: &str) -> Option<(u32, &str)> {
    v.find("filename=")
        .map(|index| &v[9 + index..])
        .map(|rest| rest.trim_matches('"'))
        .map(|s| (1, s))
}

fn from_percent_encoded(v: &str) -> Option<(u32, String)> {
    v.find("filename*=")
        .map(|index| &v[10 + index..])
        .and_then(|rest| rest.split_once("''"))
        .and_then(|(_, name)| percent_decode(name.as_bytes()).decode_utf8().ok())
        .map(|s| (0, s.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unit_filename_from_header() {
        assert_eq!(
            filename_from_header("inline; filename=\"test.jpg\""),
            Some("test.jpg".into())
        );

        assert_eq!(
            filename_from_header("inline; filename=\"XXXXXXX_XXXX_Unterj?hrige Entgeltaufstellung_vom_XX.XX.XXXX_XXXXXXXXXXXXXX.pdf\"; filename*=UTF-8''XXXXXXX_XXXX_Unterj%C3%A4hrige%20Entgeltaufstellung_vom_XX.XX.XXXX_XXXXXXXXXXXXXX.pdf"),
            Some("XXXXXXX_XXXX_Unterjährige Entgeltaufstellung_vom_XX.XX.XXXX_XXXXXXXXXXXXXX.pdf".into())
        );
    }
}
