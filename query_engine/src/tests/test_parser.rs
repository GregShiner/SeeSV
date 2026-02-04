#[cfg(test)]
mod tests {
    use crate::parser::grammar;
    use rstest::rstest;

    macro_rules! set_snapshot_suffix {
        ($($expr:expr),*) => {
            let mut settings = insta::Settings::clone_current();
            settings.set_snapshot_suffix(format!($($expr,)*));
            let _guard = settings.bind_to_scope();
        }
    }

    #[rstest]
    #[case("SELECT * FROM;")]
    #[case("SELECT FROM users;")]
    #[case("SELECT*FROMusers;")]
    #[case("* FROM users;")]
    #[case("SELECT * FROM table")]
    fn test_invalid_queries(#[case] input: &str) {
        let parser = grammar::SelectQueryParser::new();
        assert!(parser.parse(input).is_err());
    }

    #[rstest]
    #[case(1, "SELECT * FROM users;")]
    #[case(2, "SELECT column FROM users;")]
    #[case(3, r#"SELECT column FROM "test";"#)]
    #[case(4, r#"SELECT "column";"#)]
    #[case(5, r#"SELECT 5;"#)]
    #[case(6, r#"SELECT 5.5;"#)]
    #[case(7, r#"SELECT .5;"#)]
    fn test_ast(#[case] id: i32, #[case] input: &str) {
        let parser = grammar::QueryParser::new();
        let query = parser.parse(input).unwrap();

        set_snapshot_suffix!("{}", id);

        // Put the input and query in the snapshot to make it easier to see which query goes to
        // which output
        insta::assert_yaml_snapshot!((input, query));
    }
}
