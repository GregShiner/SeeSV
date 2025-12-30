#[cfg(test)]
mod tests {
    use crate::parser::grammar;
    use rstest::rstest;

    #[test]
    fn test_query_parser() {
        use crate::ast::*;

        let parser = grammar::QueryParser::new();

        assert_eq!(
            parser.parse("SELECT * FROM test;").unwrap(),
            Query::Select(SelectQuery {
                select_exprs: vec![SelectExpr::All],
                table_ref: Some(TableRef::Identifier(Identifier::from("test")))
            })
        )
    }

    #[rstest]
    #[case("SELECT * FROM users;")]
    #[case("select * from users;")]
    #[case("SELECT  *  FROM  users;")]
    fn test_valid_queries(#[case] input: &str) {
        let parser = grammar::SelectQueryParser::new();
        assert!(parser.parse(input).is_ok());
    }

    #[rstest]
    #[case("SELECT * FROM;")]
    #[case("SELECT FROM users;")]
    #[case("SELECT*FROMusers;")]
    #[case("* FROM users;")]
    fn test_invalid_queries(#[case] input: &str) {
        let parser = grammar::SelectQueryParser::new();
        assert!(parser.parse(input).is_err());
    }

    #[rstest]
    #[case("SELECT * FROM users;")]
    #[case("SELECT column FROM users;")]
    #[case(r#"SELECT column FROM "test";"#)]
    #[case(r#"SELECT "column";"#)]
    #[case(r#"SELECT 5;"#)]
    #[case(r#"SELECT 5.5;"#)]
    #[case(r#"SELECT .5;"#)]
    fn test_select_query_ast(#[case] input: &str) {
        let parser = grammar::SelectQueryParser::new();
        let query = parser.parse(input).unwrap();

        // Put the input and query in the snapshot to make it easier to see which query goes to
        // which output
        insta::assert_yaml_snapshot!((input, query));
    }
}
