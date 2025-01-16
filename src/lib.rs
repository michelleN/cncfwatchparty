use serde_json;

use http::{response, StatusCode};
use spin_sdk::http::{IntoResponse, Method, Request, Response};
use spin_sdk::http_component;
use spin_sdk::variables;

/// A simple Spin HTTP component.
#[http_component]
async fn handle_cncfwatchparty(req: Request) -> anyhow::Result<impl IntoResponse> {
    println!("Handling request to {:?}", req.header("spin-full-url"));
    // Create the outbound request object
    let token = variables::get("github_pat")?;

    let toc = [
        "linsun",
        "angellk",
        "dims",
        "TheFoxAtWork",
        "cathyhongzhang",
        "kevin-wangzefeng",
        "nikhita",
        "kgamanji",
        "rochaporto",
        "mauilion",
        "dzolotusky",
    ];

    let request = Request::builder()
        .method(Method::Get)
        .header("Accept", "application/vnd.github+json")
        .header("Authorization", format!("Bearer {}", token))
        .header("User-Agent", "cncfwatchparty")
        .uri("https://api.github.com/repos/cncf/sandbox/issues/comments/2590410827/reactions?per_page=100")
        .build();

    // Send the request and await the response
    let response: Response = spin_sdk::http::send(request).await?;

    if response.status() == &StatusCode::OK {
        // return body

        let gif_url = "https://media1.giphy.com/media/v1.Y2lkPTc5MGI3NjExd2Jnc3d3ZXB4NnY1NWgwNTl1dW52Y3U4cnE4cDg2cWNjZGdvMmhmMCZlcD12MV9pbnRlcm5hbF9naWZfYnlfaWQmY3Q9Zw/5GoVLqeAOo6PK/giphy.gif";
        // grab usernames from response
        let (usernames, toc_votes) = parse_usernames_and_update_counter(response.body(), &toc);

        // Generate the HTML content
        let html_body = format!(
            r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <meta http-equiv="refresh" content="100"> <!-- Refresh every 100 seconds -->
    <title>CNCF Watch Party</title>
    <script src="https://cdn.jsdelivr.net/npm/canvas-confetti@1.6.0/dist/confetti.browser.min.js"></script>
    <style>
        body {{
            font-family: Arial, sans-serif;
            text-align: center;
            margin-top: 50px;
        }}
        ul {{
            list-style-type: none;
            padding: 0;
        }}
    </style>
</head>
<body>
    <h1>TOC/BINDING VOTES SO FAR: {}</h1>
    <h3>Please note 8 out of the 11 TOC votes are required for acceptance.</h3>
    <img src="{}" alt="A cool GIF" />
    <h1>Binding and Nonbinding Votes: {}</h1>
    <ul>
        {}
    </ul>

    <script>
        // Number of TOC votes
        const tocVotes = {};

        // Trigger confetti if tocVotes is 8 or higher
        if (tocVotes >= 8) {{
            confetti({{
                particleCount: 100,
                spread: 70,
                origin: {{ y: 0.6 }}
            }});
        }}
    </script>
</body>
</html>
"#,
            toc_votes,       // Injecting the number of TOC votes
            gif_url,         // Injecting the GIF URL
            usernames.len(), // Injecting the count of usernames
            usernames
                .iter()
                .map(|username| format!("<li>{}</li>", username))
                .collect::<String>(), // Generating list items for usernames
            toc_votes        // Reusing toc_votes for the JavaScript logic
        );

        return Ok(Response::builder()
            .status(200)
            .header("content-type", "text/html")
            .body(html_body)
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

fn parse_usernames_and_update_counter(body: &[u8], toc_usernames: &[&str]) -> (Vec<String>, u32) {
    let mut toc_votes = 0;
    let mut usernames = Vec::new();
    let body_str = std::str::from_utf8(&body).unwrap();
    let json: serde_json::Value = serde_json::from_str(body_str).unwrap();
    for reaction in json.as_array().unwrap() {
        if reaction["content"].as_str().unwrap() != "+1" {
            continue;
        }
        let username = reaction["user"]["login"].as_str().unwrap();
        if toc_usernames.contains(&username) {
            toc_votes += 1;
        }
        usernames.push(username.to_string());
    }
    (usernames, toc_votes)
}
