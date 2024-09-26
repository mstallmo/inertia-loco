//! Default Loco Initializer provided for adding InertiaJS rendering to Loco apps
//!
//! If modifications are needed to the behavior of the initializer a custom initializer
//! can be implemented in your Loco app directly.
use crate::InertiaConfigBuilder;
use axum::{async_trait, Extension, Router as AxumRouter};
use loco_rs::{
    app::{AppContext, Initializer},
    Error, Result,
};
use tracing::debug;

pub struct InertiaInitializer;

#[async_trait]
impl Initializer for InertiaInitializer {
    fn name(&self) -> String {
        "inertia".to_string()
    }

    /// Creates a new [InertiaConfig] instance based on the [AppContext] environment and adds
    /// it to the router as a layer [Extension]
    async fn after_routes(&self, router: AxumRouter, ctx: &AppContext) -> Result<AxumRouter> {
        debug!("Initializing...");

        let inertia_config = InertiaConfigBuilder::new(ctx.environment.clone())
            .build()
            .map_err(|err| Error::Message(err.to_string()))?;

        Ok(router.layer(Extension(inertia_config)))
    }
}
