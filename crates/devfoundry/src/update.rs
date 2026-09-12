use std::time::Duration;

const CHECK_TIMEOUT: Duration = Duration::from_secs(2);
const DEFAULT_ENDPOINT: &str = "https://api.github.com/repos/<OWNER>/<REPOSITORY>/releases/latest";
const ENDPOINT_ENV: &str = "DEVFOUNDRY_UPDATE_ENDPOINT";
const DISABLE_ENV: &str = "DEVFOUNDRY_NO_UPDATE_CHECK";
const MAX_METADATA_BYTES: usize = 64 * 1024;

#[derive(Debug, serde::Deserialize)]
struct Release {
    tag_name: String,
    html_url: String,
}

pub async fn check() {
    if !cfg!(target_os = "macos") || std::env::var_os(DISABLE_ENV).is_some() {
        return;
    }

    let endpoint = std::env::var(ENDPOINT_ENV).unwrap_or_else(|_| DEFAULT_ENDPOINT.to_owned());
    if endpoint == DEFAULT_ENDPOINT || !valid_endpoint(&endpoint) {
        return;
    }

    let client = match reqwest::Client::builder()
        .timeout(CHECK_TIMEOUT)
        .user_agent(concat!("devfoundry/", env!("CARGO_PKG_VERSION")))
        .build()
    {
        Ok(client) => client,
        Err(_) => return,
    };

    let response = match client
        .get(endpoint)
        .header(reqwest::header::ACCEPT, "application/vnd.github+json")
        .send()
        .await
        .and_then(|response| response.error_for_status())
    {
        Ok(response) => response,
        Err(_) => return,
    };
    if response
        .content_length()
        .is_some_and(|length| length > MAX_METADATA_BYTES as u64)
    {
        return;
    }
    let body = match response.bytes().await {
        Ok(body) if body.len() <= MAX_METADATA_BYTES => body,
        _ => return,
    };
    let release: Release = match serde_json::from_slice(&body) {
        Ok(release) => release,
        Err(_) => return,
    };

    if !valid_tag(&release.tag_name) || !valid_release_url(&release.html_url) {
        return;
    }

    let current = match semver::Version::parse(env!("CARGO_PKG_VERSION")) {
        Ok(current) => current,
        Err(_) => return,
    };
    let latest = match semver::Version::parse(release.tag_name.trim_start_matches('v')) {
        Ok(latest) => latest,
        Err(_) => return,
    };
    if latest <= current {
        return;
    }

    eprintln!(
        "devfoundry update available: {} {}",
        release.tag_name, release.html_url
    );
}

fn valid_tag(tag: &str) -> bool {
    let version = tag.strip_prefix('v').unwrap_or(tag);
    !version.is_empty()
        && version.len() <= 64
        && version
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, '.' | '-'))
}

fn valid_release_url(url: &str) -> bool {
    reqwest::Url::parse(url).is_ok_and(|url| {
        url.scheme() == "https"
            && url.host_str() == Some("github.com")
            && url.username().is_empty()
            && url.password().is_none()
            && url.query().is_none()
            && url.fragment().is_none()
    })
}

fn valid_endpoint(endpoint: &str) -> bool {
    reqwest::Url::parse(endpoint).is_ok_and(|url| {
        matches!(url.scheme(), "http" | "https")
            && url.host_str().is_some()
            && url.username().is_empty()
            && url.password().is_none()
            && url.query().is_none()
            && url.fragment().is_none()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn release_fields_are_strictly_safe() {
        assert!(valid_tag("v1.2.3"));
        assert!(!valid_tag("v1.2.3\nsecret"));
        assert!(valid_release_url(
            "https://github.com/example/project/releases/tag/v1.2.3"
        ));
        assert!(!valid_release_url(
            "https://github.com/example/project?token=secret"
        ));
        assert!(!valid_release_url(
            "https://user:pass@github.com/example/project"
        ));
        assert!(!valid_release_url("http://github.com/example/project"));
        assert!(valid_endpoint(
            "https://api.github.com/repos/example/project/releases/latest"
        ));
        assert!(!valid_endpoint(
            "https://api.github.com/releases/latest?token=secret"
        ));
        assert!(!valid_endpoint("file:///tmp/metadata"));
    }
}
