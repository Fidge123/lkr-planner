const REDACTED: &str = "[redacted]";
const SENSITIVE_KEYS: [&str; 8] = [
    "token",
    "password",
    "passwort",
    "secret",
    "key",
    "authorization",
    "auth",
    "refresh_token",
];

/// Whitespace-delimited so the surrounding description survives redaction.
pub fn sanitize(message: &str) -> String {
    let mut sanitized: Vec<String> = Vec::new();
    let mut redact_next = false;

    for word in message.split_whitespace() {
        let (prefix, core, suffix) = split_punctuation(word);

        if redact_next {
            redact_next = false;
            if !core.is_empty() {
                sanitized.push(format!("{prefix}{REDACTED}{suffix}"));
                continue;
            }
        }

        if core.eq_ignore_ascii_case("bearer") || core.eq_ignore_ascii_case("basic") {
            redact_next = true;
            sanitized.push(word.to_string());
            continue;
        }

        sanitized.push(match classify(core) {
            Some(replacement) => format!("{prefix}{replacement}{suffix}"),
            None => word.to_string(),
        });
    }

    sanitized.join(" ")
}

fn classify(core: &str) -> Option<String> {
    if core.is_empty() {
        return None;
    }

    if core.contains("://") {
        return Some(REDACTED.to_string());
    }

    if is_email(core) {
        return Some(REDACTED.to_string());
    }

    if is_absolute_path(core) {
        return Some(REDACTED.to_string());
    }

    redact_sensitive_pair(core)
}

fn is_email(core: &str) -> bool {
    match core.split_once('@') {
        Some((local, domain)) => {
            !local.is_empty() && domain.contains('.') && !domain.ends_with('.')
        }
        None => false,
    }
}

fn is_absolute_path(core: &str) -> bool {
    (core.starts_with('/') || core.starts_with("~/")) && core.matches('/').count() >= 2
}

fn redact_sensitive_pair(core: &str) -> Option<String> {
    let (key, value) = core.split_once('=')?;
    if value.is_empty() {
        return None;
    }

    let normalized = key.rsplit(['.', '-', '_']).next().unwrap_or(key);
    SENSITIVE_KEYS
        .iter()
        .any(|sensitive| normalized.eq_ignore_ascii_case(sensitive))
        .then(|| format!("{key}={REDACTED}"))
}

fn split_punctuation(word: &str) -> (&str, &str, &str) {
    let is_edge = |character: char| matches!(character, '(' | ')' | '[' | ']' | '\'' | '"' | ',');
    let start = word.len() - word.trim_start_matches(is_edge).len();
    let trimmed = word[start..].trim_end_matches(|character| {
        is_edge(character) || matches!(character, '.' | ':' | ';' | '!' | '?')
    });

    (&word[..start], trimmed, &word[start + trimmed.len()..])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redacts_urls_and_keeps_the_description() {
        let sanitized = sanitize("CalDAV PUT https://app.zep.de/caldav/admin/emp-1/x.ics failed");

        assert!(!sanitized.contains("zep.de"));
        assert!(sanitized.contains("CalDAV PUT"));
        assert!(sanitized.contains("failed"));
    }

    #[test]
    fn redacts_absolute_file_paths() {
        let sanitized = sanitize("Datei konnte nicht gelesen werden (/Users/flori/config.json)");

        assert!(!sanitized.contains("flori"));
        assert!(sanitized.contains("Datei konnte nicht gelesen werden"));
    }

    #[test]
    fn redacts_bearer_tokens() {
        let sanitized = sanitize("Authorization: Bearer eyJhbGciOiJIUzI1NiJ9.abc rejected");

        assert!(!sanitized.contains("eyJhbGciOiJIUzI1NiJ9"));
        assert!(sanitized.contains("rejected"));
    }

    #[test]
    fn redacts_email_addresses() {
        let sanitized = sanitize("Login für flori@hey.com abgelehnt");

        assert!(!sanitized.contains("flori@hey.com"));
        assert!(sanitized.contains("abgelehnt"));
    }

    #[test]
    fn redacts_sensitive_key_value_pairs() {
        let sanitized = sanitize("request failed token=s3cret password=hunter2");

        assert!(!sanitized.contains("s3cret"));
        assert!(!sanitized.contains("hunter2"));
        assert!(sanitized.contains("request failed"));
    }

    #[test]
    fn keeps_plain_technical_descriptions_intact() {
        let sanitized = sanitize("Nager API returned status 503");

        assert_eq!(sanitized, "Nager API returned status 503");
    }
}
