//! Parse Instagram URLs and instaloader target syntax into download targets.
//!
//! Target forms follow instaloader docs:
//! <https://instaloader.github.io/cli-options.html#targets>

use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    Empty,
    Invalid(String),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::Empty => write!(f, "请粘贴 Instagram 链接"),
            ParseError::Invalid(s) => write!(f, "无法解析链接: {s}"),
        }
    }
}

impl std::error::Error for ParseError {}

/// A single instaloader download target derived from a URL or native syntax.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DownloadTarget {
    /// Single post: `instaloader -- -SHORTCODE`
    Post(String),
    /// Profile posts: `instaloader USERNAME`
    Profile(String),
    /// Hashtag posts: `instaloader "#HASHTAG"`
    Hashtag(String),
    /// Location posts: `instaloader %LOCATION_ID`
    Location(String),
}

impl DownloadTarget {
    /// Label shown in the UI and stored on `DownloadResult.shortcode`.
    pub fn display_label(&self) -> String {
        match self {
            Self::Post(s) => s.clone(),
            Self::Profile(s) => s.clone(),
            Self::Hashtag(s) => format!("#{s}"),
            Self::Location(s) => format!("%{s}"),
        }
    }

    /// Argument passed to instaloader after `--`.
    pub fn instaloader_target(&self) -> String {
        match self {
            Self::Post(s) => format!("-{s}"),
            Self::Profile(s) => s.clone(),
            Self::Hashtag(s) => format!("#{s}"),
            Self::Location(s) => format!("%{s}"),
        }
    }

    /// Directory name instaloader creates under the destination.
    pub fn folder_name(&self) -> String {
        match self {
            Self::Post(s) => s.clone(),
            Self::Profile(s) => s.clone(),
            Self::Hashtag(s) => format!("#{s}"),
            Self::Location(s) => format!("%{s}"),
        }
    }

    pub fn is_post(&self) -> bool {
        matches!(self, Self::Post(_))
    }
}

/// Parse one Instagram URL, native instaloader target, or bare shortcode.
pub fn parse_target(input: &str) -> Result<DownloadTarget, ParseError> {
    let s = input
        .trim()
        .trim_matches(|c| c == '<' || c == '>' || c == '"' || c == '\'');
    if s.is_empty() {
        return Err(ParseError::Empty);
    }
    if let Some(target) = target_from_native(s) {
        return Ok(target);
    }
    if let Some(target) = target_from_instagram_url(s) {
        return Ok(target);
    }
    if is_shortcode(s) {
        return Ok(DownloadTarget::Post(s.to_string()));
    }
    Err(ParseError::Invalid(s.to_string()))
}

/// Parse one Instagram 短码链接 or a bare shortcode into the post shortcode.
pub fn parse_shortcode(input: &str) -> Result<String, ParseError> {
    match parse_target(input)? {
        DownloadTarget::Post(code) => Ok(code),
        _ => Err(ParseError::Invalid(input.trim().to_string())),
    }
}

/// Parse one or many links. Each non-empty line is one link.
pub fn parse_links(input: &str) -> Result<Vec<DownloadTarget>, ParseError> {
    let mut out = Vec::new();
    let mut saw_token = false;
    for raw in input.lines() {
        let raw = raw.trim().trim_matches(|c| c == ',' || c == ';');
        if raw.is_empty() {
            continue;
        }
        saw_token = true;
        if let Ok(target) = parse_target(raw) {
            if !out.iter().any(|existing| existing == &target) {
                out.push(target);
            }
        }
    }
    if out.is_empty() {
        if saw_token {
            Err(ParseError::Invalid(input.trim().to_string()))
        } else {
            Err(ParseError::Empty)
        }
    } else {
        Ok(out)
    }
}

fn target_from_native(s: &str) -> Option<DownloadTarget> {
    if let Some(tag) = s.strip_prefix('#') {
        let tag = percent_decode(tag);
        if is_hashtag_name(&tag) {
            return Some(DownloadTarget::Hashtag(tag));
        }
        return None;
    }
    if let Some(id) = s.strip_prefix('%') {
        if is_location_id(id) {
            return Some(DownloadTarget::Location(id.to_string()));
        }
        return None;
    }
    if let Some(code) = s.strip_prefix('-') {
        if is_shortcode(code) {
            return Some(DownloadTarget::Post(code.to_string()));
        }
        return None;
    }
    None
}

fn target_from_instagram_url(s: &str) -> Option<DownloadTarget> {
    let path = instagram_path(s)?;
    let path_only = path.split(['?', '#']).next().unwrap_or(path);
    let path_only = path_only.trim_matches('/');
    if path_only.is_empty() {
        return None;
    }
    let parts: Vec<&str> = path_only.split('/').filter(|p| !p.is_empty()).collect();
    if parts.is_empty() {
        return None;
    }
    let first = parts[0].to_ascii_lowercase();
    match first.as_str() {
        "p" | "reel" | "reels" | "tv" => {
            let code = parts.get(1)?;
            if is_shortcode(code) {
                Some(DownloadTarget::Post((*code).to_string()))
            } else {
                None
            }
        }
        "share" => share_shortcode(&parts).map(DownloadTarget::Post),
        "explore" => explore_target(&parts),
        "tags" => {
            let tag = percent_decode(parts.get(1)?);
            if is_hashtag_name(&tag) {
                Some(DownloadTarget::Hashtag(tag))
            } else {
                None
            }
        }
        "stories" => None,
        _ => username_prefixed_target(&parts),
    }
}

fn share_shortcode(parts: &[&str]) -> Option<String> {
    // /share/p/CODE, /share/reel/CODE, /share/CODE
    let code = if parts.len() >= 3 && matches!(parts[1].to_ascii_lowercase().as_str(), "p" | "reel" | "reels" | "tv")
    {
        parts[2]
    } else {
        parts.get(1).copied()?
    };
    if is_shortcode(code) {
        Some(code.to_string())
    } else {
        None
    }
}

fn explore_target(parts: &[&str]) -> Option<DownloadTarget> {
    let kind = parts.get(1)?.to_ascii_lowercase();
    match kind.as_str() {
        "tags" => {
            let tag = percent_decode(parts.get(2)?);
            if is_hashtag_name(&tag) {
                Some(DownloadTarget::Hashtag(tag))
            } else {
                None
            }
        }
        "locations" => {
            let id = *parts.get(2)?;
            if is_location_id(id) {
                Some(DownloadTarget::Location(id.to_string()))
            } else {
                None
            }
        }
        _ => None,
    }
}

fn username_prefixed_target(parts: &[&str]) -> Option<DownloadTarget> {
    if !is_username(parts[0]) {
        return None;
    }
    if parts.len() >= 3 {
        let kind = parts[1].to_ascii_lowercase();
        if matches!(kind.as_str(), "p" | "reel" | "reels" | "tv") && is_shortcode(parts[2]) {
            return Some(DownloadTarget::Post(parts[2].to_string()));
        }
    }
    Some(DownloadTarget::Profile(parts[0].to_string()))
}

fn instagram_path(s: &str) -> Option<&str> {
    let lower = s.to_ascii_lowercase();
    for marker in ["instagram.com/", "instagr.am/"] {
        if let Some(idx) = lower.find(marker) {
            let start = idx + marker.len();
            if start <= s.len() {
                return Some(&s[start..]);
            }
        }
    }
    None
}

fn is_shortcode(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 64
        && s.chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
        && s.chars().any(|c| c.is_ascii_alphanumeric())
}

fn is_username(s: &str) -> bool {
    let s = s.strip_prefix('@').unwrap_or(s);
    !s.is_empty()
        && s.len() <= 30
        && !s.starts_with('.')
        && !s.ends_with('.')
        && s.chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '.')
}

fn is_hashtag_name(s: &str) -> bool {
    !s.is_empty() && s.len() <= 128 && s.chars().all(|c| c.is_alphanumeric() || c == '_')
}

fn is_location_id(s: &str) -> bool {
    !s.is_empty() && s.len() <= 32 && s.chars().all(|c| c.is_ascii_digit())
}

fn percent_decode(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let Ok(hex) = std::str::from_utf8(&bytes[i + 1..i + 3]) {
                if let Ok(b) = u8::from_str_radix(hex, 16) {
                    out.push(b);
                    i += 3;
                    continue;
                }
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    const CODE: &str = "DdivRY4CFX6";

    #[test]
    fn parse_p_url_plain() {
        assert_eq!(
            parse_shortcode("instagram.com/p/DdivRY4CFX6").unwrap(),
            CODE
        );
    }

    #[test]
    fn parse_p_url_https_www_slash() {
        assert_eq!(
            parse_shortcode("https://www.instagram.com/p/DdivRY4CFX6/").unwrap(),
            CODE
        );
    }

    #[test]
    fn parse_p_url_http_no_www() {
        assert_eq!(
            parse_shortcode("http://instagram.com/p/DdivRY4CFX6").unwrap(),
            CODE
        );
    }

    #[test]
    fn parse_p_url_with_query_string() {
        assert_eq!(
            parse_shortcode(
                "https://www.instagram.com/p/DdivRY4CFX6/?igsh=abc123&img_index=1"
            )
            .unwrap(),
            CODE
        );
    }

    #[test]
    fn parse_reel_url_with_www() {
        assert_eq!(
            parse_shortcode("https://www.instagram.com/reel/ReelCode99/").unwrap(),
            "ReelCode99"
        );
    }

    #[test]
    fn parse_reel_url_without_www() {
        assert_eq!(
            parse_shortcode("https://instagram.com/reel/ReelCode99").unwrap(),
            "ReelCode99"
        );
    }

    #[test]
    fn parse_reels_plural_url() {
        assert_eq!(
            parse_shortcode("https://www.instagram.com/reels/ReelCode99/").unwrap(),
            "ReelCode99"
        );
    }

    #[test]
    fn parse_tv_url_with_query() {
        assert_eq!(
            parse_shortcode("https://www.instagram.com/tv/TvCode_01/?hl=en").unwrap(),
            "TvCode_01"
        );
    }

    #[test]
    fn parse_tv_url_bare_host() {
        assert_eq!(
            parse_shortcode("instagram.com/tv/TvCode_01").unwrap(),
            "TvCode_01"
        );
    }

    #[test]
    fn parse_username_prefixed_post() {
        assert_eq!(
            parse_target("https://www.instagram.com/natgeo/p/DdivRY4CFX6/").unwrap(),
            DownloadTarget::Post(CODE.into())
        );
    }

    #[test]
    fn parse_username_prefixed_reel() {
        assert_eq!(
            parse_target("https://www.instagram.com/natgeo/reel/ReelCode99/").unwrap(),
            DownloadTarget::Post("ReelCode99".into())
        );
        assert_eq!(
            parse_target("https://www.instagram.com/natgeo/reels/ReelCode99").unwrap(),
            DownloadTarget::Post("ReelCode99".into())
        );
    }

    #[test]
    fn parse_instagr_am_and_mobile_hosts() {
        assert_eq!(
            parse_shortcode("https://instagr.am/p/DdivRY4CFX6/").unwrap(),
            CODE
        );
        assert_eq!(
            parse_shortcode("https://m.instagram.com/reel/ReelCode99").unwrap(),
            "ReelCode99"
        );
    }

    #[test]
    fn parse_share_url() {
        assert_eq!(
            parse_target("https://www.instagram.com/share/p/DdivRY4CFX6/").unwrap(),
            DownloadTarget::Post(CODE.into())
        );
        assert_eq!(
            parse_target("https://www.instagram.com/share/reel/ReelCode99").unwrap(),
            DownloadTarget::Post("ReelCode99".into())
        );
    }

    #[test]
    fn parse_profile_url() {
        assert_eq!(
            parse_target("https://www.instagram.com/natgeo/").unwrap(),
            DownloadTarget::Profile("natgeo".into())
        );
        assert_eq!(
            parse_target("instagram.com/natgeo").unwrap(),
            DownloadTarget::Profile("natgeo".into())
        );
    }

    #[test]
    fn parse_hashtag_url_and_native() {
        assert_eq!(
            parse_target("https://www.instagram.com/explore/tags/kitten/").unwrap(),
            DownloadTarget::Hashtag("kitten".into())
        );
        assert_eq!(
            parse_target("#kitten").unwrap(),
            DownloadTarget::Hashtag("kitten".into())
        );
        assert_eq!(
            parse_target("https://www.instagram.com/explore/tags/%E7%8C%AB/").unwrap(),
            DownloadTarget::Hashtag("猫".into())
        );
    }

    #[test]
    fn parse_location_url_and_native() {
        assert_eq!(
            parse_target(
                "https://www.instagram.com/explore/locations/362629379/plymouth-naval-memorial/"
            )
            .unwrap(),
            DownloadTarget::Location("362629379".into())
        );
        assert_eq!(
            parse_target("%362629379").unwrap(),
            DownloadTarget::Location("362629379".into())
        );
    }

    #[test]
    fn parse_native_shortcode_dash() {
        assert_eq!(
            parse_target("-DdivRY4CFX6").unwrap(),
            DownloadTarget::Post(CODE.into())
        );
    }

    #[test]
    fn parse_bare_shortcode() {
        assert_eq!(parse_shortcode("DdivRY4CFX6").unwrap(), CODE);
        assert_eq!(parse_shortcode("  ABC-123_x  ").unwrap(), "ABC-123_x");
    }

    #[test]
    fn parse_links_batch() {
        let input = "https://www.instagram.com/p/DdivRY4CFX6/\nhttps://instagram.com/reel/ReelCode99\nTvCode_01";
        let codes = parse_links(input).unwrap();
        assert_eq!(
            codes,
            vec![
                DownloadTarget::Post("DdivRY4CFX6".into()),
                DownloadTarget::Post("ReelCode99".into()),
                DownloadTarget::Post("TvCode_01".into()),
            ]
        );
    }

    #[test]
    fn parse_links_mixed_formats() {
        let input = "https://www.instagram.com/p/DdivRY4CFX6/\nhttps://www.instagram.com/natgeo/\nhttps://www.instagram.com/explore/tags/kitten/\n%362629379";
        let targets = parse_links(input).unwrap();
        assert_eq!(
            targets,
            vec![
                DownloadTarget::Post("DdivRY4CFX6".into()),
                DownloadTarget::Profile("natgeo".into()),
                DownloadTarget::Hashtag("kitten".into()),
                DownloadTarget::Location("362629379".into()),
            ]
        );
    }

    #[test]
    fn parse_links_newline_separated_with_blank_lines_and_crlf() {
        let input = "https://www.instagram.com/p/DdivRY4CFX6/\r\n\r\nhttps://www.instagram.com/reel/ReelCode99/\n\ninstagram.com/tv/TvCode_01\n";
        let codes = parse_links(input).unwrap();
        assert_eq!(
            codes,
            vec![
                DownloadTarget::Post("DdivRY4CFX6".into()),
                DownloadTarget::Post("ReelCode99".into()),
                DownloadTarget::Post("TvCode_01".into()),
            ]
        );
    }

    #[test]
    fn instaloader_target_strings_match_docs() {
        assert_eq!(
            DownloadTarget::Post(CODE.into()).instaloader_target(),
            "-DdivRY4CFX6"
        );
        assert_eq!(
            DownloadTarget::Profile("natgeo".into()).instaloader_target(),
            "natgeo"
        );
        assert_eq!(
            DownloadTarget::Hashtag("kitten".into()).instaloader_target(),
            "#kitten"
        );
        assert_eq!(
            DownloadTarget::Location("362629379".into()).instaloader_target(),
            "%362629379"
        );
    }

    #[test]
    fn reject_non_instagram_url() {
        assert!(parse_shortcode("https://example.com/p/DdivRY4CFX6").is_err());
    }

    #[test]
    fn reject_empty() {
        assert_eq!(parse_shortcode("  "), Err(ParseError::Empty));
        assert_eq!(parse_links(""), Err(ParseError::Empty));
    }
}
