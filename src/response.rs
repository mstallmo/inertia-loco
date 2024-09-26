use crate::config::InertiaConfig;
use crate::{page::Page, request::Request};
use axum::response::{Html, IntoResponse, Json};
use http::HeaderMap;

/// An Inertia response.
///
/// More information at:
/// [https://inertiajs.com/the-protocol#inertia-responses](https://inertiajs.com/the-protocol#inertia-responses)
pub struct Response {
    pub(crate) request: Request,
    pub(crate) page: Page,
    pub(crate) config: InertiaConfig,
}

impl IntoResponse for Response {
    fn into_response(self) -> axum::response::Response {
        let mut headers = HeaderMap::new();
        if let Some(version) = &self.config.version() {
            headers.insert("X-Inertia-Version", version.parse().unwrap());
        }

        if self.request.is_xhr {
            headers.insert("X-Inertia", "true".parse().unwrap());
            (headers, Json(self.page)).into_response()
        } else {
            let html = self.config.layout(&self.page).unwrap();
            (headers, Html(html)).into_response()
        }
    }
}

#[cfg(test)]
mod tests {
    use http_body_util::BodyExt;
    use loco_rs::environment::Environment;

    use crate::InertiaConfigBuilder;

    use super::*;

    #[tokio::test]
    async fn test_into_html_response() {
        let request = Request {
            is_xhr: false,
            ..Request::test_request()
        };
        let page = Page {
            component: "Testing",
            props: serde_json::json!({ "test": "test" }),
            url: "/test".to_string(),
            version: None,
        };

        let config = InertiaConfigBuilder::new(Environment::Development)
            .views_dir(&"test-assets")
            .build()
            .unwrap();

        let response = Response {
            request,
            page,
            config,
        }
        .into_response();
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let body = String::from_utf8(body.into()).expect("decoded string");

        // Since tera makes the HTML safe we have to check against `&quot;` instead of a literal "
        assert!(body.contains(r#"&quot;props&quot;:{&quot;test&quot;:&quot;test&quot;}"#));
    }
}
