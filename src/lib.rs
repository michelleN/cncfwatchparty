use serde_json;

use http::StatusCode;
use spin_sdk::http::{IntoResponse, Method, Request, Response};
use spin_sdk::http_component;
use spin_sdk::variables;

/// A simple Spin HTTP component.
#[http_component]
async fn handle_cncfwatchparty(req: Request) -> anyhow::Result<impl IntoResponse> {
    println!("Handling request to {:?}", req.header("spin-full-url"));
    // Create the outbound request object
    let token = variables::get("github_pat")?;

    let request = Request::builder()
        .method(Method::Get)
        .header("Accept", "application/vnd.github+json")
        .header("Authorization", format!("Bearer {}", token))
        .header("User-Agent", "cncfwatchparty")
        .uri("https://api.github.com/repos/cncf/sandbox/issues/comments/2590410827/reactions")
        .build();

    // Send the request and await the response
    let response: Response = spin_sdk::http::send(request).await?;

    if response.status() == &StatusCode::OK {
        // return body

        // grab usernames from response
        let usernames = parse_usernames(response.body()).join("\n");
        return Ok(Response::builder()
            .status(200)
            .header("content-type", "text/plain")
            .body(usernames)
            .build());
    }
    println!("Error: {:?}", response.status());
    println!("Body: {:?}", std::str::from_utf8(response.body()).unwrap());
    // return error
    Ok(Response::builder()
        .status(500)
        .header("content-type", "text/plain")
        .body("Something bad happened")
        .build())
}

fn parse_usernames(body: &[u8]) -> Vec<String> {
    let mut usernames = Vec::new();
    let body_str = std::str::from_utf8(&body).unwrap();
    let json: serde_json::Value = serde_json::from_str(body_str).unwrap();
    for reaction in json.as_array().unwrap() {
        let username = reaction["user"]["login"].as_str().unwrap();
        usernames.push(username.to_string());
    }
    usernames
}
