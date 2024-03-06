use itertools::Itertools;
use crate::*;


#[derive(Default, Clone, PartialEq, Debug)]
struct TestCaseConversion {
    line_number: usize,
    ehr_entry: String,
    ehr_entry_plus: String,
    snellen_equivalent: String,
    log_mar_equivalent: String,
    log_mar_plus_letters: String,
    comment: String,
}

impl TestCaseConversion {
    fn load() -> Vec<TestCaseConversion> {
        include_str!("../../../../testing/test_cases_conversions.tsv")
            .lines()
            .map(|l| l.split('\t').map(|s| s.to_string()).collect_tuple().unwrap())
            .enumerate()
            .skip(1)
            .map(|(n, (
                ehr_entry, ehr_entry_plus, snellen_equivalent, log_mar_equivalent,
                log_mar_plus_letters, comment,
              ))| TestCaseConversion {
                line_number: n + 1, ehr_entry, ehr_entry_plus, snellen_equivalent,
                log_mar_equivalent, log_mar_plus_letters, comment,
            })
            .filter(|tc| !tc.ehr_entry.starts_with('#'))
            .collect_vec()
    }
}

#[test]
fn test_cases_conversions() {
    let parser = Parser::new();
    for test_case in TestCaseConversion::load() {
        fn unwrap<T, E>(obj: &Result<Option<T>, E>, func: fn(&T) -> String) -> String {
            match obj { Ok(Some(f)) => func(f), _ => "Error".to_string() }
        }

        fn format_float (f: &f64) -> String {
            if f > &0.0 { format!("+{f:.2}") } else { format!("{f:.2}") }
        }

        let input = [
            ("EHR Entry", test_case.ehr_entry.clone()),
            ("EHR Entry Plus", test_case.ehr_entry_plus.clone()),
        ];

        let parsed = parser.parse_visit(input.into()).unwrap();
        let note = parsed.0.get("EHR Entry").unwrap().clone();
        let actual = TestCaseConversion {
            line_number: test_case.line_number.clone(),
            comment: test_case.comment.clone(),
            ehr_entry: note.text,
            ehr_entry_plus: note.text_plus,
            snellen_equivalent: unwrap(&note.snellen_equivalent, |s| s.to_string()),
            log_mar_equivalent: unwrap(&note.log_mar_base , format_float),
            log_mar_plus_letters: unwrap(&note.log_mar_base_plus_letters , format_float),
        };

        assert_eq!(actual, test_case, "(actual == expected)");
    }
}

