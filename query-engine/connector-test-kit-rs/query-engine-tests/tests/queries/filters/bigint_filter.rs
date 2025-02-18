use super::common_test_data;
use query_engine_tests::*;

#[test_suite(schema(schemas::common_nullable_types), exclude(Sqlite("libsql.js.wasm")))]
mod bigint_filter_spec {
    use query_engine_tests::run_query;

    #[connector_test]
    async fn basic_where(runner: Runner) -> TestResult<()> {
        common_test_data(&runner).await?;

        insta::assert_snapshot!(
          run_query!(&runner, r#"query { findManyTestModel(where: { bInt: { equals: "5" }}) { id }}"#),
          @r###"{"data":{"findManyTestModel":[{"id":1}]}}"###
        );

        insta::assert_snapshot!(
          run_query!(&runner, r#"query { findManyTestModel(where: { AND: [{ bInt: { not: "1" }}, { bInt: { not: null }}] }) { id }}"#),
          @r###"{"data":{"findManyTestModel":[{"id":1}]}}"###
        );

        insta::assert_snapshot!(
          run_query!(&runner, r#"query { findManyTestModel(where: { bInt: { not: null }}) { id }}"#),
          @r###"{"data":{"findManyTestModel":[{"id":1},{"id":2}]}}"###
        );

        Ok(())
    }

    #[connector_test]
    async fn where_shorthands(runner: Runner) -> TestResult<()> {
        common_test_data(&runner).await?;

        insta::assert_snapshot!(
          run_query!(&runner, r#"query { findManyTestModel(where: { bInt: "5" }) { id }}"#),
          @r###"{"data":{"findManyTestModel":[{"id":1}]}}"###
        );

        match_connector_result!(
          &runner,
          r#"query { findManyTestModel(where: { bInt: null }) { id }}"#,
          // MongoDB excludes undefined fields
          MongoDb(_) => vec![r#"{"data":{"findManyTestModel":[]}}"#],
          _ => vec![r#"{"data":{"findManyTestModel":[{"id":3}]}}"#]
        );

        Ok(())
    }

    #[connector_test]
    async fn inclusion_filter(runner: Runner) -> TestResult<()> {
        common_test_data(&runner).await?;

        insta::assert_snapshot!(
          run_query!(&runner, r#"query { findManyTestModel(where: { bInt: { in: ["5", "1"] }}) { id }}"#),
          @r###"{"data":{"findManyTestModel":[{"id":1},{"id":2}]}}"###
        );

        insta::assert_snapshot!(
          run_query!(&runner, r#"query { findManyTestModel(where: { AND: [ { bInt: { notIn: ["1"] }}, { bInt: { not: null }} ]}) { id }}"#),
          @r###"{"data":{"findManyTestModel":[{"id":1}]}}"###
        );

        insta::assert_snapshot!(
          run_query!(&runner, r#"query { findManyTestModel(where: { AND: [{ bInt: { not: { in: ["1"] }}}, { bInt: { not: null }}] }) { id }}"#),
          @r###"{"data":{"findManyTestModel":[{"id":1}]}}"###
        );

        Ok(())
    }

    #[connector_test]
    async fn numeric_comparison_filters(runner: Runner) -> TestResult<()> {
        common_test_data(&runner).await?;

        // Gt
        insta::assert_snapshot!(
          run_query!(&runner, r#"query { findManyTestModel(where: { bInt: { gt: "1" }}) { id }}"#),
          @r###"{"data":{"findManyTestModel":[{"id":1}]}}"###
        );

        // Not gt => lte
        insta::assert_snapshot!(
          run_query!(&runner, r#"query { findManyTestModel(where: { bInt: { not: { gt: "1" }}}) { id }}"#),
          @r###"{"data":{"findManyTestModel":[{"id":2}]}}"###
        );

        // Gte
        insta::assert_snapshot!(
          run_query!(&runner, r#"query { findManyTestModel(where: { bInt: { gte: "1" }}) { id }}"#),
          @r###"{"data":{"findManyTestModel":[{"id":1},{"id":2}]}}"###
        );

        // Not gte => lt
        insta::assert_snapshot!(
          run_query!(&runner, r#"query { findManyTestModel(where: { bInt: { not: { gte: "5" }}}) { id }}"#),
          @r###"{"data":{"findManyTestModel":[{"id":2}]}}"###
        );

        // Lt
        insta::assert_snapshot!(
          run_query!(&runner, r#"query { findManyTestModel(where: { bInt: { lt: "6" }}) { id }}"#),
          @r###"{"data":{"findManyTestModel":[{"id":1},{"id":2}]}}"###
        );

        // Not lt => gte
        insta::assert_snapshot!(
          run_query!(&runner, r#"query { findManyTestModel(where: { bInt: { not: { lt: "5" }}}) { id }}"#),
          @r###"{"data":{"findManyTestModel":[{"id":1}]}}"###
        );

        // Lte
        insta::assert_snapshot!(
          run_query!(&runner, r#"query { findManyTestModel(where: { bInt: { lte: "5" }}) { id }}"#),
          @r###"{"data":{"findManyTestModel":[{"id":1},{"id":2}]}}"###
        );

        // Not lte => gt
        insta::assert_snapshot!(
          run_query!(&runner, r#"query { findManyTestModel(where: { bInt: { not: { lte: "1" }}}) { id }}"#),
          @r###"{"data":{"findManyTestModel":[{"id":1}]}}"###
        );

        Ok(())
    }
}
