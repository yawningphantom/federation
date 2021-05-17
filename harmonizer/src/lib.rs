/*!
# Harmonizer

This _harmonizer_ offers the ability to invoke a bundled version of the
JavaScript library, [`@apollo/federation`], which _composes_ multiple subgraphs
into a supergraph.

The bundled version of the federation library that is included is a JavaScript
Immediately Invoked Function Expression ([IIFE]) that is created by running the
[Rollup.js] bundler on the `@apollo/federation` package.

When the [`harmonize`] function that this crate provides is called with a
[`ServiceList`] (which is synonymous with the terminology and service list
notion that exists within the JavaScript composition library), this crate uses
[`quick-js`] to invoke the JavaScript in the [QuickJS Engine].  We previously
attempted to do this using V8 (via [`deno_core`] and [`rusty_v8`]), but that
resulted in problems running on Musl-based C libraries.

While we intend for a future version of composition to be done natively within
Rust, this allows us to provide a more stable transition using an already stable
composition implementation while we work toward something else.

[`@apollo/federation`]: https://npm.im/@apollo/federation
[IIFE]: https://developer.mozilla.org/en-US/docs/Glossary/IIFE
[Rollup.js]: http://rollupjs.org/
[`quick-js`]: https://crates.io/crates/quick-js
[QuickJS Engine]: https://bellard.org/quickjs/
[`deno_core`]: https://crates.io/crates/deno_core
*/

#![forbid(unsafe_code)]
#![deny(missing_debug_implementations, nonstandard_style)]
#![warn(missing_docs, future_incompatible, unreachable_pub, rust_2018_idioms)]
use quick_js::{Context, JsValue, ValueError};
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use thiserror::Error;

/// The `ServiceDefinition` represents everything we need to know about a
/// service (subgraph) for its GraphQL runtime responsibilities.  It is not
/// at all different from the notion of [`ServiceDefinition` in TypeScript]
/// used in Apollo Gateway's operation.
///
/// Since we'll be running this within a JavaScript environment these properties
/// will be serialized into camelCase, to match the JavaScript expectations.
///
/// [`ServiceDefinition` in TypeScript]: https://github.com/apollographql/federation/blob/d2e34909/federation-js/src/composition/types.ts#L49-L53
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ServiceDefinition {
    /// The name of the service (subgraph).  We use this name internally to
    /// in the representation of the composed schema and for designations
    /// within the human-readable QueryPlan.
    pub name: String,
    /// The routing/runtime URL where the subgraph can be found that will
    /// be able to fulfill the requests it is responsible for.
    pub url: String,
    /// The Schema Definition Language (SDL)
    pub type_defs: String,
}

impl ServiceDefinition {
    /// Create a new [`ServiceDefinition`]
    pub fn new<N: Into<String>, U: Into<String>, D: Into<String>>(
        name: N,
        url: U,
        type_defs: D,
    ) -> ServiceDefinition {
        ServiceDefinition {
            name: name.into(),
            url: url.into(),
            type_defs: type_defs.into(),
        }
    }
}

/// An ordered stack of the services (subgraphs) that, when composed in order
/// by the composition algorithm, will represent the supergraph.
pub type ServiceList = Vec<ServiceDefinition>;

/// An error which occurred during JavaScript composition.
///
/// The shape of this error is meant to mimick that of the error created within
/// JavaScript, which is a [`GraphQLError`] from the [`graphql-js`] library.
///
/// [`graphql-js']: https://npm.im/graphql
/// [`GraphQLError`]: https://github.com/graphql/graphql-js/blob/3869211/src/error/GraphQLError.js#L18-L75
#[derive(Debug, Error, Serialize, Deserialize, PartialEq)]
pub struct CompositionError {
    /// A human-readable description of the error that prevented composition.
    pub message: Option<String>,
    /// [`CompositionErrorExtensions`]
    pub extensions: Option<CompositionErrorExtensions>,
}

impl Display for CompositionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(msg) = &self.message {
            f.write_fmt(format_args!("{code}: {msg}", code = self.code(), msg = msg))
        } else {
            f.write_str(self.code())
        }
    }
}

#[derive(Debug, Error, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
/// The result of running composition, possibly with errors.
///
/// Either of the properties may be present or absent.  It's expected that
/// without `errors` there be a `supergraph_sdl`.
pub struct CompositionResult {
    /// If present, there were errors during composition
    // pub errors: Option<Vec<CompositionError>>,
    /// This is the string representation of the supergraph (core) schema.
    pub supergraph_sdl: Option<String>,
}

impl Display for CompositionResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(_result) = &self.supergraph_sdl  {
            f.write_str("")
        } else {
            f.write_fmt(format_args!("composition error?"))
        }
    }
}


// impl From<CompositionResult> for JsValue {
//     fn from(value: CompositionResult) -> Self {
//         JsValue::String(value.supergraph_sdl)
//     }
// }

impl std::convert::TryFrom<JsValue> for CompositionResult {
    type Error = ValueError;

    fn try_from(value: JsValue) -> Result<Self, Self::Error> {
        match value {
            JsValue::Object(_inner) => Ok(CompositionResult {
                supergraph_sdl: Some(String::from("fuck")),
                // errors: Some(vec![]),
            }),
            _ => Err(ValueError::UnexpectedType)
        }

    }
}

// impl Display for CompositionResult {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         if let Some(msg) = &self.message {
//             f.write_fmt(format_args!("{code}: {msg}", code = self.code(), msg = msg))
//         } else {
//             f.write_str(self.code())
//         }
//     }
// }

/// Mimicking the JavaScript-world from which this error comes, this represents
/// the `extensions` property of a JavaScript [`GraphQLError`] from the
/// [`graphql-js`] library. Such errors are created when errors have prevented
/// successful composition, which is accomplished using [`errorWithCode`]. An
/// [example] of this can be seen within the `federation-js` JavaScript library.
///
/// [`graphql-js']: https://npm.im/graphql
/// [`GraphQLError`]: https://github.com/graphql/graphql-js/blob/3869211/src/error/GraphQLError.js#L18-L75
/// [`errorWithCode`]: https://github.com/apollographql/federation/blob/d7ca0bc2/federation-js/src/composition/utils.ts#L200-L216
/// [example]: https://github.com/apollographql/federation/blob/d7ca0bc2/federation-js/src/composition/validate/postComposition/executableDirectivesInAllServices.ts#L47-L53
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct CompositionErrorExtensions {
    /// An Apollo Federation composition error code.
    ///
    /// A non-exhaustive list of error codes that this includes, is:
    ///
    ///   - EXTERNAL_TYPE_MISMATCH
    ///   - EXTERNAL_UNUSED
    ///   - KEY_FIELDS_MISSING_ON_BASE
    ///   - KEY_MISSING_ON_BASE
    ///   - KEY_NOT_SPECIFIED
    ///   - PROVIDES_FIELDS_MISSING_EXTERNAL
    ///
    /// ...and many more!  See the `federation-js` composition library for
    /// more details (and search for `errorWithCode`).
    pub code: String,
}

/// An error that was received during composition within JavaScript.
impl CompositionError {
    /// Retrieve the error code from an error received during composition.
    pub fn code(&self) -> &str {
        match self.extensions {
            Some(ref ext) => &*ext.code,
            None => "UNKNOWN",
        }
    }
}

/// The `harmonize` function receives a [`ServiceList`] and invokes JavaScript
/// composition on it.
///
pub fn harmonize(service_list: ServiceList) -> Result<String, Vec<CompositionError>> {
    // Initialize a runtime instance
    let context = Context::builder()
        .console(quick_js::console::LogConsole)
        .build()
        .unwrap();

    context
        .eval(
            r#"
// We build some of the preliminary objects that our Rollup-built package is
// expecting to be present in the environment.
// node_fetch_1 is an unused external dependency we don't bundle.  See the
// configuration in this package's 'rollup.config.js' for where this is marked
// as an external dependency and thus not packaged into the bundle.
node_fetch_1 = {};
// 'process' is a Node.js ism.  We rely on process.env.NODE_ENV, in
// particular, to determine whether or not we are running in a debug
// mode.  For the purposes of harmonizer, we don't gain anything from
// running in such a mode.
process = { env: { "NODE_ENV": "production" }};
// Some JS runtime implementation specific bits that we rely on that
// need to be initialized as empty objects.
global = {};
exports = {};
    "#,
        )
        .expect("unable to initialize composition runtime environment");

    // Load the composition library.
    context
        .eval(include_str!("../dist/composition.js"))
        .expect("unable to evaluate composition module");

    // We turn services into a JSON object that we'll execute in the runtime
    let service_list_javascript = format!(
        "serviceList = {}",
        serde_json::to_string(&service_list)
            .expect("unable to serialize service list into JavaScript runtime")
    );

    context
        .eval(&service_list_javascript)
        .expect("unable to evaluate service list in JavaScript runtime");

    context
        .eval(include_str!("../js/do_compose.js"))
        .expect("unable to invoke composition in JavaScript runtime");

    let value: String = context
        .eval_as(r#"
          // Save only what we can reasonably deserialize.
          const { errors, supergraphSdl } = composition.composeAndValidate(serviceList);
          // Return just that.
          { supergraphSdl };
          supergraphSdl || "";
        "#)
        .expect("unable to invoke composition");

    // match value.supergraph_sdl {
    Ok(value)
    // match value {
    //     Some(value) => Ok(value),
    //     None => Ok(String::from("maybe"))
    // }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        use crate::{harmonize, ServiceDefinition};

        insta::assert_snapshot!(harmonize(vec![
            ServiceDefinition::new(
                "users",
                "undefined",
                "
            type User {
              id: ID
              name: String
            }

            type Query {
              users: [User!]
            }
          "
            ),
            ServiceDefinition::new(
                "movies",
                "undefined",
                "
            type Movie {
              title: String
              name: String
            }

            extend type User {
              favorites: [Movie!]
            }

            type Query {
              movies: [Movie!]
            }
          "
            )
        ])
        .unwrap());
    }
}
