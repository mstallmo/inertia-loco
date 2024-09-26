//! An implementation of the [inertia.js] protocol for [loco].
//!
//! The basic idea is that any axum handler that accepts the `Inertia`
//! struct as a function parameter is an inertia endpoint. For
//! instance:
//!
//! ```rust ignore
//! use inertia_loco::Inertia;
//! use loco_rs::prelude::*;
//! use serde_json::json;
//!
//!
//! async fn my_handler_fn(i: Inertia) -> impl IntoResponse {
//!     i.render("Pages/MyPageComponent", json!({"myPageProps": "true"}))
//! }
//! ```
//!
//! This does the following:
//!
//! - If the incoming request is the initial page load (i.e., does not
//!   have the `X-Inertia` header set to `true`), the
//!   [render](Inertia::render) method responds with an html page, which
//!   is configurable when setting up the initial Inertia state (see
//!   [Getting started](#getting-started) below).
//!
//! - Otherwise, the handler responses with the standard inertia
//!   "Page" object json, with the included component and page props
//!   passed to [render](Inertia::render).
//!
//! - If the request has a mismatching asset version (again, this is
//!   configurable), the handler responds with a `409 Conflict` to tell
//!   the client to reload the page. The function body of the handler is
//!   not executed in this case.
//!
//! # Getting started
//!
//! First, you'll need to provide your loco routes with
//! [InertiaConfig] extension. This state boils down to two things: an
//! optional string representing the [asset version] and an instance
//! of a templating engine that will render the configured layout file.
//!
//! The [initializer] module provides a convenient way to set up the
//! [InertiaConfig] extension. If custom configuration for Inertia is
//! reqiured it is recommended that you implement your own custom loco
//! initializer in your application.
//!
//! See [https://loco.rs/docs/extras/pluggability/#initializers](https://loco.rs/docs/extras/pluggability/#initializers)
//! for more information on Loco initializers.
//!
//! The [Inertia] struct is then available as an axum [Extractor] and
//! can be used in handlers like so:
//!
//! ```rust ignore
//! use loco::prelude::*;
//! use inertia_loco::Inertia;
//! use serde_json::json;
//!
//! async fn get_root(i: Inertia) -> impl IntoResponse {
//!     i.render("Pages/Home", json!({ "posts": vec!["post one", "post two"] }))
//! }
//! ```
//!
//! The [Inertia::render] method takes care of building a response
//! conforming to the [inertia.js protocol]. It takes two parameters:
//! the name of the component to render, and the page props
//! (serializable to json).
//!
//! [asset version]: https://inertiajs.com/the-protocol#asset-versioning
//! [inertia.js]: https://inertiajs.com
//! [inertia.js protocol]: https://inertiajs.com/the-protocol
//! [loco]: https://crates.io/crates/loco
//! [Extractor]: https://docs.rs/axum/latest/axum/#extractors
//! [Initializer]: https://loco.rs/docs/extras/pluggability/#initializers

use async_trait::async_trait;
use axum::{extract::FromRequestParts, Extension};
pub use config::{InertiaConfig, InertiaConfigBuilder};
use http::{request::Parts, HeaderMap, HeaderValue, StatusCode};
pub use in_vite;
use page::Page;
use props::Props;
use request::Request;
use response::Response;

pub mod config;
pub mod initializer;
mod page;
pub mod partial;
pub mod props;
mod request;
mod response;
mod tera;

#[derive(Clone)]
pub struct Inertia {
    request: Request,
    config: InertiaConfig,
}

#[async_trait]
impl<S> FromRequestParts<S> for Inertia
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, HeaderMap<HeaderValue>);

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        use axum::RequestPartsExt;
        let Extension(config) =
            parts
                .extract::<Extension<InertiaConfig>>()
                .await
                .map_err(|_err| {
                    // TODO: log error to conosle
                    (StatusCode::INTERNAL_SERVER_ERROR, HeaderMap::new())
                })?;

        let request = Request::from_request_parts(parts, state).await?;

        // Respond with a 409 conflict if X-Inertia-Version values
        // don't match for GET requests. See more at:
        // https://inertiajs.com/the-protocol#asset-versioning
        if parts.method == "GET"
            && request.is_xhr
            && config.version().is_some()
            && request.version != config.version()
        {
            let mut headers = HeaderMap::new();
            headers.insert("X-Inertia-Location", parts.uri.path().parse().unwrap());
            return Err((StatusCode::CONFLICT, headers));
        }

        Ok(Inertia::new(request, config))
    }
}

impl Inertia {
    fn new(request: Request, config: InertiaConfig) -> Inertia {
        Inertia { request, config }
    }

    /// Renders an Inertia response.
    pub fn render<S: Props>(self, component: &'static str, props: S) -> Response {
        let request = self.request;
        let url = request.url.clone();
        let page = Page {
            component,
            props: props
                .serialize(request.partial.as_ref())
                // TODO: error handling
                .expect("serialization failure"),
            url,
            version: self.config.version().clone(),
        };
        Response {
            page,
            request,
            config: self.config,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{self, response::IntoResponse, routing::get, Router};
    use loco_rs::environment::Environment;
    use reqwest::StatusCode;
    use serde_json::json;
    use tokio::net::TcpListener;

    #[tokio::test]
    async fn it_works() {
        async fn handler(i: Inertia) -> impl IntoResponse {
            i.render("foo!", json!({"bar": "baz"}))
        }

        let config = InertiaConfigBuilder::new(Environment::Development)
            .views_dir(&"test-assets")
            .build()
            .unwrap();

        let app = Router::new()
            .route("/test", get(handler))
            .layer(Extension(config));

        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("Could not bind ephemeral socket");
        let addr = listener.local_addr().unwrap();

        tokio::spawn(async move {
            axum::serve(listener, app).await.expect("server error");
        });

        let res = reqwest::get(format!("http://{}/test", &addr))
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn it_responds_with_conflict_on_version_mismatch() {
        async fn handler(i: Inertia) -> impl IntoResponse {
            i.render("foo!", json!({"bar": "baz"}))
        }

        let config = InertiaConfigBuilder::new(Environment::Production)
            .views_dir(&"test-assets")
            .vite_manifest_path(&"test-assets/.dist/manifest.json")
            .build()
            .unwrap();

        let app = Router::new()
            .route("/test", get(handler))
            .layer(Extension(config));

        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("Could not bind ephemeral socket");
        let addr = listener.local_addr().unwrap();

        tokio::spawn(async move {
            axum::serve(listener, app).await.expect("server error");
        });

        let client = reqwest::Client::new();

        let res = client
            .get(format!("http://{}/test", &addr))
            .header("X-Inertia", "true")
            .header("X-Inertia-Version", "456")
            .send()
            .await
            .unwrap();

        assert_eq!(res.status(), StatusCode::CONFLICT);
        assert_eq!(
            res.headers()
                .get("X-Inertia-Location")
                .map(|h| h.to_str().unwrap()),
            Some("/test")
        );
    }
}
