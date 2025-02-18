mod mongodb_renderer;
mod sql_renderer;

pub use mongodb_renderer::*;
pub use sql_renderer::*;

use crate::{
    connection_string, templating, ConnectorVersion, DatamodelFragment, IdFragment, M2mFragment, MySqlVersion, CONFIG,
};
use indoc::indoc;
use itertools::Itertools;
use once_cell::sync::Lazy;
use psl::{PreviewFeature, ALL_PREVIEW_FEATURES};
use regex::Regex;

/// Test configuration, loaded once at runtime.
static FRAGMENT_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"#.*").unwrap());

/// The main trait a datamodel renderer for a connector has to implement.
pub trait DatamodelRenderer {
    fn render(&self, fragment: DatamodelFragment) -> String {
        match fragment {
            DatamodelFragment::Id(id) => self.render_id(id),
            DatamodelFragment::M2m(m2m) => self.render_m2m(m2m),
        }
    }

    fn render_id(&self, id: IdFragment) -> String;
    fn render_m2m(&self, m2m: M2mFragment) -> String;
}

/// Render the complete datamodel with all bells and whistles.
pub fn render_test_datamodel(
    test_database: &str,
    template: String,
    excluded_features: &[&str],
    relation_mode_override: Option<String>,
    db_schemas: &[&str],
    isolation_level: Option<&'static str>,
) -> String {
    let (tag, version) = CONFIG.test_connector().unwrap();
    let preview_features = render_preview_features(excluded_features, &version);

    let is_multi_schema = !db_schemas.is_empty();

    let schema_def = if is_multi_schema {
        format!("schemas = {db_schemas:?}")
    } else {
        String::default()
    };

    let datasource_with_generator = format!(
        indoc! {r#"
            datasource test {{
                provider = "{}"
                url = "{}"
                relationMode = "{}"
                {}
            }}

            generator client {{
                provider = "prisma-client-js"
                previewFeatures = [{}]
            }}
        "#},
        tag.datamodel_provider(),
        connection_string(&CONFIG, &version, test_database, is_multi_schema, isolation_level),
        relation_mode_override.unwrap_or_else(|| tag.relation_mode().to_string()),
        schema_def,
        preview_features
    );

    let renderer = tag.datamodel_renderer();
    let models = process_template(template, renderer);

    format!("{datasource_with_generator}\n\n{models}")
}

fn process_template(template: String, renderer: Box<dyn DatamodelRenderer>) -> String {
    let mut fragment_defs = vec![];

    for cap in FRAGMENT_RE.captures_iter(&template) {
        let fragment = templating::parse(&cap[0]).unwrap(); // todo error handling
        fragment_defs.push(fragment);
    }

    let preprocessed = FRAGMENT_RE.replace_all(&template, "#{}");

    fragment_defs.into_iter().fold(preprocessed.to_string(), |aggr, next| {
        aggr.replacen("#{}", &renderer.render(next), 1)
    })
}

fn render_preview_features(excluded_features: &[&str], version: &ConnectorVersion) -> String {
    let mut excluded_features: Vec<_> = excluded_features.iter().map(|f| format!(r#""{f}""#)).collect();

    // TODO: Remove this once we are able to have version speficic preview features.
    if version.is_mysql() && !matches!(version, ConnectorVersion::MySql(Some(MySqlVersion::V8))) {
        excluded_features.push(format!(r#""{}""#, PreviewFeature::RelationJoins));
    }

    ALL_PREVIEW_FEATURES
        .active_features()
        .iter()
        .chain(ALL_PREVIEW_FEATURES.hidden_features())
        .map(|f| format!(r#""{f}""#))
        .filter(|f| !excluded_features.contains(f))
        .join(", ")
}
