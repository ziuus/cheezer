use base64::Engine;
use std::error::Error;

pub async fn create_remediation_pr(
    file_path: &str,
    new_content: &str,
    pr_title: &str,
    pr_body: &str,
) -> Result<String, Box<dyn Error + Send + Sync>> {
    let token = match std::env::var("GITHUB_TOKEN") {
        Ok(t) if !t.trim().is_empty() => t.trim().to_string(),
        _ => return Err("GITHUB_TOKEN environment variable is missing or empty".into()),
    };
    let repo = match std::env::var("GITHUB_REPO") {
        Ok(r) if !r.trim().is_empty() => r.trim().to_string(),
        _ => return Err("GITHUB_REPO environment variable is missing or empty".into()),
    };

    let base_url = std::env::var("GITHUB_API_URL").unwrap_or_else(|_| "https://api.github.com".to_string());

    let client = reqwest::Client::builder()
        .user_agent("cheezer-operator")
        .default_headers({
            let mut headers = reqwest::header::HeaderMap::new();
            headers.insert(
                reqwest::header::AUTHORIZATION,
                reqwest::header::HeaderValue::from_str(&format!("Bearer {}", token))?,
            );
            headers.insert(
                reqwest::header::ACCEPT,
                reqwest::header::HeaderValue::from_static("application/vnd.github+json"),
            );
            headers
        })
        .build()?;

    // Step c: GET /repos/{repo}/git/ref/heads/main
    let ref_url = format!("{}/repos/{}/git/ref/heads/main", base_url, repo);
    let res = client.get(&ref_url).send().await?;
    if !res.status().is_success() {
        let err_text = res.text().await?;
        return Err(format!("Failed to get main branch ref: {}", err_text).into());
    }
    let ref_json: serde_json::Value = res.json().await?;
    let main_sha = ref_json["object"]["sha"]
        .as_str()
        .ok_or("Missing object.sha in GitHub ref response")?
        .to_string();

    // Step d: POST /repos/{repo}/git/refs
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let branch_name = format!("cheezer/fix-{}", timestamp);
    let branch_ref = format!("refs/heads/{}", branch_name);

    let create_ref_url = format!("{}/repos/{}/git/refs", base_url, repo);
    let create_ref_payload = serde_json::json!({
        "ref": branch_ref,
        "sha": main_sha
    });
    let res = client.post(&create_ref_url).json(&create_ref_payload).send().await?;
    if !res.status().is_success() {
        let err_text = res.text().await?;
        return Err(format!("Failed to create branch {}: {}", branch_ref, err_text).into());
    }

    // Step e: GET /repos/{repo}/contents/{file_path}?ref=main
    let contents_url = format!("{}/repos/{}/contents/{}?ref=main", base_url, repo, file_path);
    let res = client.get(&contents_url).send().await?;
    let file_sha = if res.status().is_success() {
        let contents_json: serde_json::Value = res.json().await?;
        contents_json["sha"].as_str().map(|s| s.to_string())
    } else if res.status() == reqwest::StatusCode::NOT_FOUND {
        None
    } else {
        let err_text = res.text().await?;
        return Err(format!("Failed to check content for {}: {}", file_path, err_text).into());
    };

    // Step f: Base64-encode new_content
    let encoded_content = base64::engine::general_purpose::STANDARD.encode(new_content.as_bytes());

    // Step g: PUT /repos/{repo}/contents/{file_path}
    let put_contents_url = format!("{}/repos/{}/contents/{}", base_url, repo, file_path);
    let mut put_payload = serde_json::json!({
        "message": pr_title,
        "content": encoded_content,
        "branch": branch_name
    });
    if let Some(sha) = file_sha {
        put_payload["sha"] = serde_json::Value::String(sha);
    }
    let res = client.put(&put_contents_url).json(&put_payload).send().await?;
    if !res.status().is_success() {
        let err_text = res.text().await?;
        return Err(format!("Failed to update file {}: {}", file_path, err_text).into());
    }

    // Step h: POST /repos/{repo}/pulls
    let pr_url = format!("{}/repos/{}/pulls", base_url, repo);
    let pr_payload = serde_json::json!({
        "title": pr_title,
        "body": pr_body,
        "head": branch_name,
        "base": "main"
    });
    let res = client.post(&pr_url).json(&pr_payload).send().await?;
    if !res.status().is_success() {
        let err_text = res.text().await?;
        return Err(format!("Failed to create PR: {}", err_text).into());
    }

    // Step i: Parse response JSON and extract html_url
    let pr_res_json: serde_json::Value = res.json().await?;
    let html_url = pr_res_json["html_url"]
        .as_str()
        .ok_or("Missing html_url in GitHub PR response")?
        .to_string();

    log::info!("Successfully created GitHub PR: {}", html_url);
    Ok(html_url)
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn test_create_remediation_pr_success() {
        let mock_server = MockServer::start().await;

        // 1. GET /repos/owner/repo/git/ref/heads/main
        Mock::given(method("GET"))
            .and(path("/repos/owner/repo/git/ref/heads/main"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "ref": "refs/heads/main",
                "object": { "sha": "1234567890abcdef" }
            })))
            .mount(&mock_server)
            .await;

        // 2. POST /repos/owner/repo/git/refs
        Mock::given(method("POST"))
            .and(path("/repos/owner/repo/git/refs"))
            .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
                "ref": "refs/heads/cheezer/fix-12345"
            })))
            .mount(&mock_server)
            .await;

        // 3. GET /repos/owner/repo/contents/deploy.yaml?ref=main
        Mock::given(method("GET"))
            .and(path("/repos/owner/repo/contents/deploy.yaml"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "sha": "oldfilesha123"
            })))
            .mount(&mock_server)
            .await;

        // 4. PUT /repos/owner/repo/contents/deploy.yaml
        Mock::given(method("PUT"))
            .and(path("/repos/owner/repo/contents/deploy.yaml"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "content": { "name": "deploy.yaml" }
            })))
            .mount(&mock_server)
            .await;

        // 5. POST /repos/owner/repo/pulls
        Mock::given(method("POST"))
            .and(path("/repos/owner/repo/pulls"))
            .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
                "html_url": "https://github.com/owner/repo/pull/42"
            })))
            .mount(&mock_server)
            .await;

        std::env::set_var("GITHUB_TOKEN", "test-token-123");
        std::env::set_var("GITHUB_REPO", "owner/repo");
        std::env::set_var("GITHUB_API_URL", mock_server.uri());

        let res = create_remediation_pr("deploy.yaml", "new content", "Fix memory limit", "Detailed body").await;
        assert!(res.is_ok(), "Expected Ok PR URL, got: {:?}", res);
        assert_eq!(res.unwrap(), "https://github.com/owner/repo/pull/42");

        std::env::remove_var("GITHUB_TOKEN");
        std::env::remove_var("GITHUB_REPO");
        std::env::remove_var("GITHUB_API_URL");
    }
}
