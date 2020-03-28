//! Integration/end-to-end tests for the transpilation process

static TEST_SPECS: &str = r#"
    humans | s: name | preferences
    ==============================
    SELECT humans.name, preferences.*
    FROM preferences
    LEFT JOIN humans ON humans.id = preferences.humanId
    LIMIT 10

    humans | s: id isBlue
    =====================
    SELECT id, isBlue
    FROM humans
    LIMIT 10

    humans | preferences
    ====================
    SELECT preferences.*
    FROM preferences
    LEFT JOIN humans ON humans.id = preferences.humanId
    LIMIT 10

    humans | friendMap | friendshipLog
    ==================================
    SELECT friendshipLog.*
    FROM friendshipLog
    LEFT JOIN friendMap ON friendMap.id = friendshipLog.friendshipId
    LEFT JOIN humans ON humans.id = friendMap.friendA
    LIMIT 10

    humans | u: id isBlue
    =====================
    SELECT name, email, age
    FROM humans
    LIMIT 10

    humans | w: id > 3 id < 3 id = 3 id != 3 id <= 3 id >= 3
    ========================================================
    SELECT *
    FROM humans
    WHERE id > 3 AND id < 3 AND id = 3 AND id != 3 AND id <= 3 AND id >= 3
    LIMIT 10

    humans | s: count(id) name
    =====================
    SELECT count(id), name
    FROM humans
    LIMIT 10

    humans | s: count(id) name | g: name
    ===============================
    SELECT count(id), name
    FROM humans
    GROUP BY name
    LIMIT 10

    s: 1
    ====
    SELECT 1

    humans | s: count(id) name | u: count(id)
    =========================================
    SELECT name
    FROM humans
    LIMIT 10

    humans | g: count(id)
    =====================
    SELECT *, count(id)
    FROM humans
    GROUP BY count(id)
    LIMIT 10

    humans | w: id > count(id)
    ==========================
    SELECT *
    FROM humans
    WHERE id > count(id)
    LIMIT 10
"#;

#[test]
fn test_transpiler() {
    use crate::pine_transpiler::Transpiler;

    let tests = aux::parse_tests(TEST_SPECS);
    let transpiler = aux::demo_transpiler();

    for (pine, expected_query) in &tests[..] {
        let result = transpiler.transpile(pine.as_ref() as &str);

        match result {
            Ok(generated_query) => {
                assert_eq!(
                    expected_query,
                    &generated_query,
                    "\n\nFailed to parse:\n\x1b[0;1;32m{}\x1b[0m\n\nExpected:\n\x1b[0;32m{}\x1b[0m\n\nFound:\n\x1b[0;31m{}\x1b[0m\n\n",
                    pine,
                    expected_query,
                    generated_query
                );
            }
            Err(error) => {
                assert!(
                    false,
                    "Expected to be able to parse expression:\n\x1b[0;1;32m{}\x1b[0m\n\x1b[0;31m{}\x1b[0m\n",
                    pine,
                    error
                );
            }
        }
    }
}

mod aux {
    use crate::pine_transpiler::demo::transpiler_for;
    use crate::sql::structure::{ForeignKey, Table};
    use crate::MySqlTranspiler;
    use regex::Regex;

    pub fn demo_transpiler() -> MySqlTranspiler {
        let tables = vec![
            Table {
                name: "humans".to_string(),
                columns: vec![
                    "id".into(),
                    "name".into(),
                    "email".into(),
                    "isBlue".into(),
                    "age".into(),
                ],
                foreign_keys: Vec::new(),
            },
            Table {
                name: "preferences".to_string(),
                columns: vec![
                    "id".into(),
                    "humanId".into(),
                    "tag".into(),
                    "value".into(),
                    "createdAt".into(),
                    "updatedAt".into(),
                ],
                foreign_keys: vec![fk("humanId", ("humans", "id"))],
            },
            Table {
                name: "friendMap".to_string(),
                columns: vec![
                    "id".into(),
                    "friendA".into(),
                    "friendB".into(),
                    "type".into(),
                    "createdAt".into(),
                    "updatedAt".into(),
                ],
                foreign_keys: vec![
                    fk("friendA", ("humans", "id")),
                    fk("friendB", ("humans", "id")),
                ],
            },
            Table {
                name: "friendshipLog".to_string(),
                columns: vec![
                    "id".into(),
                    "friendshipId".into(),
                    "time".into(),
                    "data".into(),
                ],
                foreign_keys: vec![fk("friendshipId", ("friendMap", "id"))],
            },
        ];

        transpiler_for(tables)
    }

    fn fk(from_column: &str, to: (&str, &str)) -> ForeignKey {
        (&(from_column, to)).into()
    }

    pub fn parse_tests(specs: &str) -> Vec<(String, String)> {
        let specs = specs.trim().replace("    ", "");

        let pine_and_expected_sql_splitter = Regex::new("\n==+\n").unwrap();
        let tests = specs
            .split("\n\n")
            .map(|spec| {
                let pieces = pine_and_expected_sql_splitter
                    .split(spec)
                    .collect::<Vec<&str>>();

                let pine = pieces[0].to_owned();
                let expected_query = pieces[1].to_owned();

                (pine, expected_query)
            })
            .collect::<Vec<(String, String)>>();

        tests
    }
}
