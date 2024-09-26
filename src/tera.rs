//! Support for using the [Tera] templating engine to render the server side
//! aspects of InertiaJS
use maud::html;
use std::collections::HashMap;
use tera::{to_value, Function, Result, Value};

/// Renders the "root" tag for InertiaJS applications to be mounted to.
/// This tag is a div with the id "app". By default InertiaJs looks for
/// this element to be present on the page when rendering the page for
/// the first time and mounts the JS application as a child of that
/// div.
///
/// See [InertiaJS Docs](https://inertiajs.com/client-side-setup#defining-a-root-element) for details.
pub(crate) struct InertiaRootTag;

impl Function for InertiaRootTag {
    fn is_safe(&self) -> bool {
        true
    }

    fn call(&self, args: &HashMap<String, Value>) -> Result<Value> {
        let Some(props) = args.get("props") else {
            return Err("Missing argument `props`. Add props to function `inertia_root(props=<inertia props>)`".into());
        };

        if !props.is_object() {
            return Err(
                format!("`props` argument should be a JSON object, got {:#?}", props).into(),
            );
        }

        let inertia_root_tag = html! {
            div #app data-page=(props) {}
        }
        .into_string();

        Ok(to_value(inertia_root_tag)?)
    }
}
