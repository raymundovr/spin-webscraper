use anyhow::Result;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use spin_sdk::{
    http::{Request, Response},
    http_component,
};
use url::Url;

#[derive(Debug, Deserialize)]
struct CrawlerPayload {
    url: String,
}

#[derive(Debug, Serialize)]
struct CrawlerResponse {
    title: String,
    description: String,
}

/// A simple Spin HTTP component.
#[http_component]
fn webscraper(req: Request) -> Result<Response> {
    let body = req.body().clone().unwrap_or_default();
    let payload: CrawlerPayload = serde_json::from_slice(&body)?;

    let url = Url::parse(&payload.url)?;

    println!("Making request to {}", url);

    let res = spin_sdk::http::send(
        http::Request::builder()
            .method("GET")
            .uri(url.to_string())
            .body(None)?,
    )?;

    let response = match res.body() {
        Some(bytes) => {
            // some websites have invalid utf-8 content
            let html_doc = unsafe { std::str::from_utf8_unchecked(bytes) };
            let html = Html::parse_document(html_doc);
            let title_selector = Selector::parse("title").unwrap();
            let title = match html.select(&title_selector).next() {
                Some(title) => title.inner_html(),
                None => "".to_string(),
            };

            let desc_selector = Selector::parse(r#"meta[name="description"]"#).unwrap();
            let description = match html.select(&desc_selector).next() {
                Some(description) => description.value().attr("content").unwrap_or("").to_string(),
                None => "".to_string(),
            };

            CrawlerResponse {
                title,
                description,
            }
        }
        _ => CrawlerResponse {
            title: String::from(""),
            description: String::from(""),
        },
    };

    let res = serde_json::to_string(&response)?;

    Ok(http::Response::builder()
        .status(200)
        .body(Some(res.into()))?)
}
