use crate::tera::InertiaRootTag;
use anyhow::{anyhow, Result};
use hex::encode;
use in_vite::{Vite, ViteMode, ViteOptions, ViteReactRefresh};
use loco_rs::environment::Environment;
use serde_json::to_value;
use sha1::{Digest, Sha1};
use std::{
    fs::read,
    path::{Path, PathBuf},
    sync::Arc,
};
use tera::Tera;

const VIEWS_DIR: &str = "assets/views";

struct Inner {
    #[allow(dead_code)]
    environment: Environment,
    version: Option<String>,
    tera: tera::Tera,
    application_layout: String,
}

#[derive(Clone)]
pub struct InertiaConfig {
    inner: Arc<Inner>,
}

impl InertiaConfig {
    /// Constructs a new InertiaConfig object.
    ///
    /// `layout` provides information about how to render the initial
    /// page load. See the [crate::vite] module for an implementation
    /// of this for vite.
    pub fn new(
        environment: Environment,
        views_dir: PathBuf,
        vite_manifest_path: PathBuf,
        application_layout: String,
        version: Option<String>,
    ) -> Result<InertiaConfig> {
        let mut tera = Self::init_tera(&views_dir)?;
        Self::init_vite(&environment, &mut tera, &vite_manifest_path);
        Self::register_inertia_root(&mut tera);

        let inner = Inner {
            environment,
            version,
            tera,
            application_layout,
        };

        Ok(InertiaConfig {
            inner: Arc::new(inner),
        })
    }

    fn init_tera(views_dir: &Path) -> Result<Tera> {
        if !views_dir.exists() {
            return Err(anyhow!(
                "missing views directory: `{}`",
                views_dir.display()
            ));
        }

        let tera = Tera::new(
            views_dir
                .join("**")
                .join("*.html")
                .to_str()
                .ok_or_else(|| anyhow!("invalid blob"))?,
        )?;

        Ok(tera)
    }

    fn init_vite(environment: &Environment, tera: &mut Tera, manifest_path: &Path) {
        let vite_mode = match environment {
            Environment::Production => ViteMode::Production,
            _ => ViteMode::Development,
        };

        let opts = ViteOptions::default().mode(vite_mode).manifest_path(
            manifest_path
                .to_str()
                .expect("Failed to convert path to str"),
        );
        let vite = Vite::with_options(opts);
        let vite_react_refresh = ViteReactRefresh::new(vite.host());

        tera.register_function("vite", vite);
        tera.register_function("vite_react_refresh", vite_react_refresh);
    }

    fn register_inertia_root(tera: &mut Tera) {
        let inertia_root_tag = InertiaRootTag {};
        tera.register_function("inertia_root", inertia_root_tag);
    }

    /// Returns a cloned optional version string.
    pub fn version(&self) -> Option<String> {
        self.inner.version.clone()
    }

    /// Returns the rendered application layout.
    pub fn layout<S: serde::Serialize + Clone>(&self, props: S) -> Result<String> {
        let mut context = tera::Context::new();
        context.insert("props", &to_value(props)?);

        let renderd_html = self
            .inner
            .tera
            .render(&self.inner.application_layout, &context)?;
        Ok(renderd_html)
    }
}

pub struct InertiaConfigBuilder {
    environment: Environment,
    views_dir: PathBuf,
    application_layout: String,
    vite_manifest_path: PathBuf,
}

impl InertiaConfigBuilder {
    /// Creates a new instance with common Loco defaults set
    /// views_dir: assets/views
    /// application_layou: layout.html
    /// vite_manifest_path: frontend/dist/.vite/manifest.json
    pub fn new(environment: Environment) -> Self {
        InertiaConfigBuilder {
            environment,
            views_dir: PathBuf::from(VIEWS_DIR),
            application_layout: "layout.html".to_string(),
            vite_manifest_path: PathBuf::from("frontend/dist/.vite/manifest.json"),
        }
    }

    /// Sets the environment that [InertiaConfig] should be built for
    pub fn environment(mut self, environment: Environment) -> Self {
        self.environment = environment;
        self
    }

    /// Sets the directory to render view templates from.
    pub fn views_dir<S: AsRef<str>>(mut self, views_dir: &S) -> Self {
        self.views_dir = PathBuf::from(views_dir.as_ref());
        self
    }

    /// Sets the template file to use as the application layout. This is the template
    /// where javascript modules and the inertia root will be placed into.
    pub fn application_layout<S: AsRef<str>>(mut self, application_layout: &S) -> Self {
        self.application_layout = application_layout.as_ref().to_string();
        self
    }

    /// Sets the path to vite manifest.json file
    pub fn vite_manifest_path<S: AsRef<str>>(mut self, manifest_path: &S) -> Self {
        self.vite_manifest_path = PathBuf::from(manifest_path.as_ref());
        self
    }

    /// Builds a new instance of [InertiaConfig]
    pub fn build(self) -> Result<InertiaConfig> {
        match self.environment {
            Environment::Development => InertiaConfig::new(
                self.environment,
                self.views_dir,
                self.vite_manifest_path,
                self.application_layout,
                None,
            ),
            _ => {
                let version = self.hash_manifest()?;
                InertiaConfig::new(
                    self.environment,
                    self.views_dir,
                    self.vite_manifest_path,
                    self.application_layout,
                    Some(version),
                )
            }
        }
    }
}

impl InertiaConfigBuilder {
    fn hash_manifest(&self) -> Result<String> {
        let manifest_bytes = read(&self.vite_manifest_path)?;
        let mut hasher = Sha1::new();
        hasher.update(manifest_bytes);
        let result = hasher.finalize();
        Ok(encode(result))
    }
}
