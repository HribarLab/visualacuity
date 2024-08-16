use crate::helpers::RoundPlaces;
use crate::tests::tsv_reader::TestCaseRow;
use crate::VisualAcuityError::*;
use crate::*;

#[test]
fn test_cases_conversions() -> VisualAcuityResult<()> {
    let parser = Parser::new();
    let content = include_str!("../../../../testing/test_cases_conversions.tsv");
    for row in TestCaseRow::read_file(content) {
        let input = [
            ("EHR Entry", row.get("EHR Entry")?),
            ("EHR Entry Plus", row.get("EHR Entry Plus")?),
        ]
        .into();

        let parsed = parser.parse_visit(input).unwrap();
        let (_, note) = parsed.into_iter().next().unwrap();
        let note = note.expect("TEST");

        let expected = (
            (
                "Snellen Equivalent",
                row.parse("Snellen Equivalent").map_err(|_| GenericError),
            ),
            (
                "LogMAR Equivalent",
                row.parse("LogMAR Equivalent").map_err(|_| GenericError),
            ),
            (
                "LogMAR Plus Letters",
                row.parse("LogMAR Plus Letters").map_err(|_| GenericError),
            ),
        );
        let actual = (
            (
                "Snellen Equivalent",
                note.snellen_equivalent.map_err(|_| GenericError),
            ),
            (
                "LogMAR Equivalent",
                note.log_mar_base.round_places(2).map_err(|_| GenericError),
            ),
            (
                "LogMAR Plus Letters",
                note.log_mar_base_plus_letters
                    .round_places(2)
                    .map_err(|_| GenericError),
            ),
        );

        assert_eq!(actual, expected, "(actual == expected)");
    }
    Ok(())
}

#[test]
fn test_cases_parsing() -> VisualAcuityResult<()> {
    let parser = Parser::new();
    let content = include_str!("../../../../testing/test_cases_parsing.tsv");
    for row in TestCaseRow::read_file(content) {
        let input: VisitInput = [
            ("EHR Entry", row.get("EHR Entry")?),
            ("EHR Entry Plus", row.get("EHR Entry Plus")?),
        ]
        .into();

        let parsed = parser.parse_visit(input.clone()).unwrap();
        let (_, note) = parsed.into_iter().next().unwrap();
        let note = note.expect("TEST");

        let expected = (
            ("Data Quality", row.get("Data Quality")?),
            ("Format", row.get("Format")?),
            ("Extracted Value", row.get("Extracted Value")?),
            (
                "Plus Letters",
                format!("[{}]", row.get("Plus Letters")?.replace('+', "")),
            ),
        );

        let VisitNote {
            data_quality,
            va_format,
            extracted_value,
            plus_letters,
            ..
        } = note;
        let actual = (
            ("Data Quality", format!("{data_quality:?}")),
            (
                "Format",
                match va_format {
                    Err(_) => s!("Error"),
                    Ok(value) => format!("{value:?}"),
                },
            ),
            ("Extracted Value", extracted_value),
            ("Plus Letters", format!("{plus_letters:?}")),
        );

        assert_eq!(
            actual, expected,
            "{} {input:?}\n(actual ?== expected)",
            row.line_number
        );
    }
    Ok(())
}
