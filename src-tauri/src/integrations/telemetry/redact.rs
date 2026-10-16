const REDACTED: &str = "[redacted]";
/// A credential under any other key name is transmitted.
const CREDENTIAL_NAMES: [&str; 9] = [
    "token",
    "password",
    "passwort",
    "passwd",
    "secret",
    "key",
    "apikey",
    "authorization",
    "auth",
];
const AUTHORIZATION_SCHEMES: [&str; 2] = ["bearer", "basic"];

pub fn sanitize(message: &str) -> String {
    redact_credential_values(&redact_url_credentials(&redact_authorization_values(
        message,
    )))
}

fn redact_authorization_values(message: &str) -> String {
    let mut redact_next = false;
    message
        .split_inclusive(char::is_whitespace)
        .map(|piece| {
            let word = piece.trim_end();
            let (prefix, core, suffix) = split_punctuation(word);
            if core.is_empty() {
                return piece.to_string();
            }
            if redact_next {
                redact_next = false;
                return format!("{prefix}{REDACTED}{suffix}{}", &piece[word.len()..]);
            }
            redact_next = is_authorization_scheme(core);
            piece.to_string()
        })
        .collect()
}

fn redact_url_credentials(message: &str) -> String {
    let mut redacted = String::with_capacity(message.len());
    let mut rest = message;
    while let Some(index) = rest.find("://") {
        let (before, after) = rest.split_at(index + 3);
        redacted.push_str(before);
        let authority_end = after
            .find(|character: char| {
                character.is_whitespace()
                    || matches!(character, '/' | '?' | '#' | ')' | ']' | '"' | '\'' | '>')
            })
            .unwrap_or(after.len());
        let authority = &after[..authority_end];
        match authority.rfind('@') {
            Some(at) => {
                redacted.push_str(REDACTED);
                redacted.push_str(&authority[at..]);
            }
            None => redacted.push_str(authority),
        }
        rest = &after[authority_end..];
    }
    redacted.push_str(rest);
    redacted
}

fn redact_credential_values(message: &str) -> String {
    let mut redacted = String::with_capacity(message.len());
    let mut position = 0;
    while let Some(character) = message[position..].chars().next() {
        if !is_identifier_character(character) {
            redacted.push(character);
            position += character.len_utf8();
            continue;
        }

        let rest = &message[position..];
        let identifier_length = rest
            .find(|candidate: char| !is_identifier_character(candidate))
            .unwrap_or(rest.len());
        let value = is_credential_name(&rest[..identifier_length])
            .then(|| credential_value(&rest[identifier_length..]))
            .flatten();
        match value {
            Some((start, end)) => {
                redacted.push_str(&rest[..identifier_length + start]);
                redacted.push_str(REDACTED);
                position += identifier_length + end;
            }
            None => {
                redacted.push_str(&rest[..identifier_length]);
                position += identifier_length;
            }
        }
    }
    redacted
}

/// Covers `key=value`, `key: value` and JSON's `"key":"value"`; an unterminated
/// quoted value runs to the end.
fn credential_value(after_key: &str) -> Option<(usize, usize)> {
    let mut index = usize::from(after_key.starts_with(['"', '\'']));
    index += leading_blanks(&after_key[index..]);
    if !after_key[index..].starts_with(['=', ':']) {
        return None;
    }
    index += 1;
    index += leading_blanks(&after_key[index..]);

    let value = &after_key[index..];
    if let Some(quote) = value
        .chars()
        .next()
        .filter(|first| matches!(first, '"' | '\''))
    {
        let start = index + 1;
        let end = after_key[start..]
            .find(quote)
            .map_or(after_key.len(), |offset| start + offset);
        return (end > start).then_some((start, end));
    }

    let length = value
        .find(|character: char| {
            character.is_whitespace()
                || matches!(
                    character,
                    '&' | ',' | ';' | '"' | '\'' | ')' | ']' | '}' | '<' | '>' | '#'
                )
        })
        .unwrap_or(value.len());
    let candidate = &value[..length];
    (length > 0 && !is_authorization_scheme(candidate)).then_some((index, index + length))
}

fn leading_blanks(text: &str) -> usize {
    text.len() - text.trim_start_matches([' ', '\t']).len()
}

fn is_identifier_character(character: char) -> bool {
    character.is_ascii_alphanumeric() || matches!(character, '_' | '-' | '.')
}

fn is_credential_name(identifier: &str) -> bool {
    let last_segment = identifier
        .rsplit(['.', '-', '_'])
        .next()
        .unwrap_or(identifier);
    CREDENTIAL_NAMES
        .iter()
        .any(|name| last_segment.eq_ignore_ascii_case(name))
}

fn is_authorization_scheme(word: &str) -> bool {
    AUTHORIZATION_SCHEMES
        .iter()
        .any(|scheme| word.eq_ignore_ascii_case(scheme))
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
    fn keeps_urls_paths_and_the_description() {
        let message = "Zeitüberschreitung bei /v1/projects/3001: error sending request for url (https://api.marketcircle.net/v1/projects/3001)";

        assert_eq!(sanitize(message), message);
    }

    #[test]
    fn keeps_email_addresses() {
        let message = "Login für flori@hey.com abgelehnt";

        assert_eq!(sanitize(message), message);
    }

    #[test]
    fn redacts_a_credential_in_a_url_query_and_keeps_the_rest_of_the_url() {
        assert_eq!(
            sanitize("error sending request for url (https://api.marketcircle.net/v1/personal_token/refresh_token?refresh_token=r3fr3sh&scope=all)"),
            "error sending request for url (https://api.marketcircle.net/v1/personal_token/refresh_token?refresh_token=[redacted]&scope=all)"
        );
    }

    #[test]
    fn redacts_credentials_in_a_json_body() {
        assert_eq!(
            sanitize(
                r#"Refresh-Antwort enthält expires_in=0. body={"access_token":"acc355","refresh_token":"r3fr3sh","expires_in":0}"#
            ),
            r#"Refresh-Antwort enthält expires_in=0. body={"access_token":"[redacted]","refresh_token":"[redacted]","expires_in":0}"#
        );
    }

    #[test]
    fn redacts_credentials_in_json_with_spaces() {
        assert_eq!(
            sanitize(r#"{ "password": "hunter2", "user": "flori" }"#),
            r#"{ "password": "[redacted]", "user": "flori" }"#
        );
    }

    #[test]
    fn redacts_a_credential_cut_off_by_truncation_to_the_end() {
        assert_eq!(
            sanitize(r#"body={"expires_in":0,"access_token":"acc355_cut..."#),
            r#"body={"expires_in":0,"access_token":"[redacted]"#
        );
    }

    #[test]
    fn redacts_credentials_in_a_url_authority() {
        assert_eq!(
            sanitize("PROPFIND https://user:pa55@app.zep.de/caldav/admin/ failed"),
            "PROPFIND https://[redacted]@app.zep.de/caldav/admin/ failed"
        );
    }

    #[test]
    fn redacts_bearer_values_and_keeps_the_scheme() {
        assert_eq!(
            sanitize("Authorization: Bearer eyJhbGciOiJIUzI1NiJ9.abc rejected"),
            "Authorization: Bearer [redacted] rejected"
        );
    }

    #[test]
    fn redacts_plain_credential_pairs() {
        assert_eq!(
            sanitize("request failed token=s3cret password=hunter2 status=401"),
            "request failed token=[redacted] password=[redacted] status=401"
        );
    }

    #[test]
    fn keeps_words_that_only_end_in_a_credential_name() {
        let message = "hotkey=F5 monkey: banana";

        assert_eq!(sanitize(message), message);
    }

    #[test]
    fn keeps_plain_technical_descriptions_intact() {
        let message = "Nager API returned status 503";

        assert_eq!(sanitize(message), message);
    }
}
