use lazy_static::lazy_static;
use std::collections::{BTreeMap, HashMap};
use test_case::test_case;

use crate::logmar::LogMarBase;
use crate::snellen_equivalent::SnellenEquivalent;
use crate::visit::*;
use crate::visitinput::*;
use crate::Correction::*;
use crate::DistanceOfMeasurement::*;
use crate::DistanceUnits::*;
use crate::Laterality::*;
use crate::VisualAcuityError::*;
use crate::*;

lazy_static! {
    static ref KEY_PARSER: KeyParser = KeyParser::new();
    static ref PLUS_LETTERS_PARSER: PlusLettersParser = PlusLettersParser::new();
    static ref JAEGER_PARSER: JaegerExactParser = JaegerExactParser::new();
    static ref SNELLEN_PARSER: SnellenExactParser = SnellenExactParser::new();
    static ref DISTANCE_UNITS_PARSER: DistanceUnitsParser = DistanceUnitsParser::new();
}

fn parse_notes(notes: &str) -> VisualAcuityResult<Vec<ParsedItem>> {
    lazy_static! {
        static ref CHART_NOTES_PARSER: ChartNotesParser = ChartNotesParser::new();
    }

    let notes = notes.trim();

    match CHART_NOTES_PARSER.parse(notes) {
        Ok(Content { content: p, .. }) => Ok(p.into_iter().collect()),
        Err(e) => Err(ParseError(format!("{e:?}: {notes}"))),
    }
}

#[test_case("EHR Entry", Ok(EntryMetadata::default()))]
#[test_case("CC VA", Ok(EntryMetadata{correction: CC, ..EntryMetadata::default() }))]
fn test_parse_key(
    notes: &str,
    expected: VisualAcuityResult<EntryMetadata>,
) -> VisualAcuityResult<()> {
    let notes = notes.trim();
    let actual = Ok(KEY_PARSER.parse(notes)?);
    assert_eq!(actual, expected, "{notes}");
    Ok(())
}

#[test]
fn test_teller() {
    let expected = vec![
        SnellenFraction(s!("20/23")),
        Teller(s!("38 cy/cm")),
        Teller(s!("Card 17")),
    ]
    .into_iter()
    .collect();
    assert_eq!(parse_notes("20/23 (38.0 cy/cm) Card 17"), Ok(expected));
    assert_eq!(
        SnellenFraction(s!("20/23")).snellen_equivalent(),
        Ok((20, 23).into()),
        "20/23 fraction"
    );
    assert_eq!(
        SnellenFraction(s!("20/23")).log_mar_base(),
        Ok(0.0606978403536117),
        "20/23 logmar"
    );
    assert_eq!(
        SnellenFraction(s!("38 cy/cm")).snellen_equivalent(),
        Ok((20, 23).into()),
        "38 cy/cm fraction"
    );
    assert_eq!(
        SnellenFraction(s!("38 cy/cm")).log_mar_base(),
        Ok(0.0606978403536117),
        "38 cy/cm logmar"
    );
    assert_eq!(
        SnellenFraction(s!("Card 17")).snellen_equivalent(),
        Ok((20, 23).into()),
        "Card 17 fraction"
    );
    assert_eq!(
        SnellenFraction(s!("Card 17")).log_mar_base(),
        Ok(0.0606978403536117),
        "Card 17 logmar"
    );
}

#[test_case(
    "CF @ 30cm",
    Ok(vec![NearTotalLoss(s!("CF"), Centimeters(30.0) )]),
    Ok((20, 1500).into()),
    Ok(1.85387196)
)]
#[test_case(
    "CF @ 3ft",
    Ok(vec![NearTotalLoss(s!("CF"), Feet(3.0) )]),
    Ok((20, 492).into()),
    Ok(1.36985702)
)]
#[test_case(
    "CF @ 8ft",
    Ok(vec![NearTotalLoss(s!("CF"), Feet(8.0) )]),
    Ok((20, 184).into()),
    Ok(0.94388828)
)]
fn test_ntlv(
    s: &str,
    expected: VisualAcuityResult<Vec<ParsedItem>>,
    expect_fraction: VisualAcuityResult<Fraction>,
    expect_log_mar: VisualAcuityResult<f64>,
) -> VisualAcuityResult<()> {
    let expected = expected.map(|exp| exp.into_iter().collect());
    let actual: VisualAcuityResult<_> = parse_notes(s).map_err(|e| UnknownError(format!("{e:?}")));
    assert_eq!(actual, expected);
    let item = actual.map(|item| item.into_iter().next().unwrap());
    assert_eq!(
        item.clone().and_then(|it| it.snellen_equivalent()),
        expect_fraction
    );
    assert_almost_eq!(
        item.clone().and_then(|it| it.log_mar_base()),
        expect_log_mar,
        8
    );
    Ok(())
}

#[test_case("+6", Ok(PlusLettersItem(6)); "plus 6 without a space")]
#[test_case("+ 6", Ok(PlusLettersItem(6)); "plus 6 with a space")]
#[test_case("-5", Ok(PlusLettersItem(- 5)))]
#[test_case("+0", Err(()))]
#[test_case("+", Err(()); "just a plus sign")]
#[test_case("-", Err(()); "just a minus sign")]
#[test_case("-7", Err(()))]
fn test_plus_letters(chart_note: &str, expected: Result<ParsedItem, ()>) {
    assert_eq!(
        PLUS_LETTERS_PARSER.parse(chart_note).map_err(|_| ()),
        expected
    );
}

#[test_case("20 / 30 + 1", Ok(vec ! [
    SnellenFraction(s!("20/30")),
    PlusLettersItem(1),
]); "20 / 30 plus one")]
#[test_case("20 / 30 - 1", Ok(vec ! [
    SnellenFraction(s!("20/30")),
    PlusLettersItem(- 1),
]); "20 / 30 minus one")]
#[test_case("20 / 30 +3 -1", Ok(vec ! [
    SnellenFraction(s!("20/30")),
    PlusLettersItem(3),
    PlusLettersItem(- 1),
]))]
#[test_case("20/30-1+6", Ok(vec ! [
    SnellenFraction(s!("20/30")),
    PlusLettersItem(- 1),
    PlusLettersItem(6),
]))]
#[test_case("20/987", Ok(vec ! [
    Text("20/987".to_string()),
]))]
#[test_case("20/321", Ok(vec ! [
    Text("20/321".to_string()),
]); "Make sure 20/321 doesn't get scooped up by 20/32")]
fn test_fractions_with_plus_letters(chart_note: &str, expected: Result<Vec<ParsedItem>, ()>) {
    let expected = expected.map(|e| e.into_iter().collect());
    assert_eq!(parse_notes(chart_note).map_err(|_| ()), expected);
}

#[test_case("J1", Ok(vec![Jaeger(s!("J1"))]))]
#[test_case("J1+", Ok(vec![Jaeger(s!("J1+"))]); "J1plus")]
#[test_case("J29", Ok(vec![Jaeger(s!("J29"))]))]
#[test_case("J0", Ok(vec![Jaeger(s!("J1+"))]); "J0")]
fn test_jaeger(chart_note: &str, expected: VisualAcuityResult<Vec<ParsedItem>>) {
    let actual = parse_notes(chart_note);
    assert_eq!(actual, expected, "{chart_note}");
}

#[test_case("CF", Ok(vec![NearTotalLoss(s!("CF"), NotProvided )]))]
#[test_case("HM", Ok(vec![NearTotalLoss(s!("HM"), NotProvided )]))]
#[test_case("LP", Ok(vec![NearTotalLoss(s!("LP"), NotProvided )]))]
#[test_case("NLP", Ok(vec![NearTotalLoss(s!("NLP"), NotProvided )]))]
#[test_case("BTL", Ok(vec![VisualResponse(s!("BTL"))]))]
#[test_case("blink to light", Ok(vec![VisualResponse(s!("BTL"))]))]
#[test_case("NI", Ok(vec![CrossReferenceItem(s!("NI"))]))]
#[test_case("CF at 1.5ft", Ok(vec![NearTotalLoss(s!("CF"), Feet(1.5) )]))]
#[test_case("CF 2'", Ok(vec![NearTotalLoss(s!("CF"), Feet(2.0) )]))]
#[test_case("CF@3'", Ok(vec![NearTotalLoss(s!("CF"), Feet(3.0) )]))]
#[test_case("CF at 3'", Ok(vec![NearTotalLoss(s!("CF"), Feet(3.0) )]))]
#[test_case("CF @ 3 feet", Ok(vec![NearTotalLoss(s!("CF"), Feet(3.0) )]))]
#[test_case("CF @ face", Ok(vec![NearTotalLoss(s!("CF"), DistanceUnits::Unhandled("CF @ face".to_string()) )]))]
#[test_case("CF @ 2M", Ok(vec![NearTotalLoss(s!("CF"), Meters(2.0) )]))]
#[test_case("CF @ 0.3 meters", Ok(vec![NearTotalLoss(s!("CF"), Meters(0.3) )]))]
#[test_case("CF @ 30 cm", Ok(vec![NearTotalLoss(s!("CF"), Centimeters(30.0) )]))]
#[test_case("No BTL", Ok(vec![VisualResponse(s!("no BTL"))]))]
#[test_case("CSM", Ok(vec![VisualResponse(s!("CSM"))]))]
#[test_case("CSM-pref", Ok(vec![VisualResponse(s!("CSM")), Text(s!("-pref"))]))]
fn test_alternative_visual_acuity(chart_note: &str, expected: Result<Vec<ParsedItem>, ()>) {
    let expected = expected.map(|e| e.into_iter().collect());
    assert_eq!(
        parse_notes(chart_note).map_err(|_| ()),
        expected,
        "{chart_note:?}"
    );
}

#[test]
fn test_distance_conversion() {
    assert_almost_eq!(Centimeters(30.0).to_feet(), Ok(0.9843), 4);
    assert_almost_eq!(Meters(0.3).to_feet(), Ok(0.9843), 4);
}

#[test_case("20/20 +1 -3", Ok(vec ! [
    SnellenFraction(s!("20/20")),
    PlusLettersItem(1),
    PlusLettersItem(- 3),
]))]
#[test_case("ok", Ok(vec ! [
Text("ok".to_string()),
]))]
#[test_case("ok 20/20 +1 -3", Ok(vec ! [
Text("ok".to_string()),
    SnellenFraction(s!("20/20")),
    PlusLettersItem(1),
    PlusLettersItem(- 3),
]))]
#[test_case("20/20 +1 -3 asdf", Ok(vec ! [
    SnellenFraction(s!("20/20")),
    PlusLettersItem(1),
    PlusLettersItem(- 3),
    Text("asdf".to_string()),
]))]
#[test_case("20/20 +1 -3 asdf qwerty", Ok(vec ! [
    SnellenFraction(s!("20/20")),
    PlusLettersItem(1),
    PlusLettersItem(- 3),
    Text("asdf qwerty".to_string()),
]))]
#[test_case("12/20 +1 -3", Ok(vec ! [
    Text("12/20".to_string()),
    PlusLettersItem(1),
    PlusLettersItem(- 3),
]))]
fn test_plain_text_notes(chart_note: &'static str, expected: VisualAcuityResult<Vec<ParsedItem>>) {
    assert_eq!(parse_notes(chart_note), expected);
}

#[test_case("20/20-1", Ok(vec ! [
    SnellenFraction(s!("20/20")),
    PlusLettersItem(- 1),
]))]
#[test_case("20/125", Ok(vec ! [
    SnellenFraction(s!("20/125")),
]))]
#[test_case("20/23 (38.0 cy/cm) Card 17", Ok(vec ! [
    SnellenFraction(s!("20/23")),
    Teller(s!("38 cy/cm")),
    Teller(s!("Card 17")),
]))]
#[test_case("20/130 (6.5 cy/cm) Card 12", Ok(vec ! [
    SnellenFraction(s!("20/130")),
    Teller(s!("6.5 cy/cm")),
    Teller(s!("Card 12")),
]))]
fn test_whole_thing(chart_note: &'static str, expected: Result<Vec<ParsedItem>, ()>) {
    assert_eq!(parse_notes(chart_note).map_err(|_| ()), expected);
}

#[test_case("fix and follow", Ok(vec ! [
    VisualResponse(s!("fix & follow"))
]))]
#[test_case("fix & follow", Ok(vec ! [
    VisualResponse(s!("fix & follow"))
]))]
#[test_case("FF", Ok(vec ! [
    VisualResponse(s!("fix & follow")),
]))]
#[test_case("F + F", Ok(vec ! [
    VisualResponse(s!("fix & follow")),
]); "F + F with plus sign and spaces")]
#[test_case("F+F", Ok(vec ! [
    VisualResponse(s!("fix & follow")),
]); "F + F with plus sign no spaces")]
#[test_case("F&F", Ok(vec ! [
    VisualResponse(s!("fix & follow")),
]); "F + F with ampersand no spaces")]
#[test_case("f/f", Ok(vec ! [
    VisualResponse(s!("fix & follow")),
]); "F + F with slash no spaces")]
#[test_case("no fix & follow", Ok(vec ! [
    VisualResponse(s!("no fix & follow")),
]))]
#[test_case("Fix, No Follow", Ok(vec ! [
    VisualResponse(s!("fix, no follow")),
]))]
#[test_case("CSM, good f+f", Ok(vec ! [
    VisualResponse(s!("CSM")),
    Text("good".to_string()),
    VisualResponse(s!("fix & follow")),
]))]
fn test_fix_and_follow(chart_note: &'static str, expected: VisualAcuityResult<Vec<ParsedItem>>) {
    assert_eq!(parse_notes(chart_note), expected, "{chart_note}");
}

#[test_case("CSM Pref", Ok(vec ! [VisualResponse(s!("CSM prefers"))]))]
#[test_case("j1", Ok(vec![Jaeger(s!("J1"))]))]
#[test_case("j30", Ok(vec ! [Text("j30".to_string())]))]
#[test_case("79 letters", Ok(vec![ETDRS(s!("79 letters"))]))]
#[test_case("81ltrs", Ok(vec![ETDRS(s!("81 letters"))]))]
#[test_case("20/13 ETDRS (95 letters)", Ok(vec![Text("20/13".to_string()), ETDRS(s!("95 letters"))]))]
#[test_case("20/20 ETDRS (83 letters)", Ok(vec![SnellenFraction(s!("20/20")), ETDRS(s!("83 letters"))]))]
fn test_various(chart_note: &'static str, expected: VisualAcuityResult<Vec<ParsedItem>>) {
    assert_eq!(parse_notes(chart_note), expected, "{chart_note}");
}

#[test_case("Both Eyes Distance CC", OU, Distance, CC, PinHole::Unknown)]
#[test_case("Manifest Both Eyes", OU, Distance, Manifest, PinHole::Unknown)]
fn test_visit_details(
    notes: &str,
    laterality: Laterality,
    distance_of_measurement: DistanceOfMeasurement,
    correction: Correction,
    pinhole: PinHole,
) -> VisualAcuityResult<()> {
    let expected = EntryMetadata {
        laterality,
        distance_of_measurement,
        correction,
        pinhole,
    };
    let actual = KEY_PARSER.parse(notes).expect("");
    assert_eq!(actual, expected);
    Ok(())
}

#[test_case("20/20", Ok(0.0))]
#[test_case("20/15", Ok(-0.1249))]
#[test_case("J2", Ok(0.0969))]
#[test_case("J1", Ok(0.0))]
#[test_case("CF", Err(NotImplemented))]
#[test_case("HM", Err(NotImplemented))]
#[test_case("LP", Err(NotImplemented))]
#[test_case("NLP", Err(NotImplemented))]
#[test_case("5 letters", Ok(1.6))]
#[test_case("10 letters", Ok(1.5))]
#[test_case("15 letters", Ok(1.4))]
#[test_case("20 letters", Ok(1.3))]
#[test_case("25 letters", Ok(1.2))]
#[test_case("30 letters", Ok(1.1))]
#[test_case("35 letters", Ok(1.0))]
#[test_case("40 letters", Ok(0.9))]
#[test_case("45 letters", Ok(0.8))]
#[test_case("50 letters", Ok(0.7))]
#[test_case("55 letters", Ok(0.6))]
#[test_case("60 letters", Ok(0.5))]
#[test_case("65 letters", Ok(0.4))]
#[test_case("70 letters", Ok(0.3))]
#[test_case("75 letters", Ok(0.2))]
#[test_case("80 letters", Ok(0.1))]
#[test_case("85 letters", Ok(0.0))]
#[test_case("90 letters", Ok(-0.1))]
#[test_case("95 letters", Ok(-0.2))]
#[test_case("58 letters", Ok(0.5); "TODO: treat n%5 as plus_letters?")]
fn test_log_mar_base(notes: &str, expected: VisualAcuityResult<f64>) -> VisualAcuityResult<()> {
    let expected: OptionResult<_> = expected.into();
    let parser = Parser::new();
    let visit_notes = [("fieldname", notes)].into();
    let parsed_notes = parser.parse_visit(visit_notes);
    let base_item = parsed_notes?
        .into_iter()
        .map(|(_, v)| v)
        .next()
        .expect("")
        .expect("");

    assert_almost_eq!(base_item.log_mar_base, expected, 4, "\"{notes}\"");
    Ok(())
}

#[test_case(
    [("", "NI")],
    Ok([("", "NI")])
)]
#[test_case(
    [("", "NT")],
    Ok([("", "NT")])
)]
#[test_case(
    [("", "unable")],
    Ok([("", "Unable")])
)]
#[test_case(
    [("", "prosthesis")],
    Ok([("", "Prosthesis")])
)]
#[test_case(
    [("", "CF 2'")],
    Ok([("", "CF @ 2 feet")])
)]
#[test_case(
    [("", "CF at 8 feet to 20/400")],
    Ok([("", "Error")])
)]
fn test_extracted_value<T: Into<VisitInput>>(visit_notes: T, expected: VisualAcuityResult<T>) {
    let parser = Parser::new();
    let visit_notes = visit_notes.into();
    let expected = expected.map(|exp| exp.into());
    let actual = parser.parse_visit(visit_notes.clone()).map(|visit| {
        visit
            .into_iter()
            .map(|(key, note)| (key, note.expect("TEST").extracted_value))
            .into()
    });
    assert_eq!(actual, expected, "{visit_notes:?}")
}

#[test_case("20/20", Ok((20, 20)))]
#[test_case("20/15", Ok((20, 15)))]
#[test_case("J1", Ok((20, 20)))]
#[test_case("J2", Ok((20, 25)))]
#[test_case("70 letters", Ok((20, 40)))]
#[test_case("58 letters", Ok((20, 63)))]
#[test_case("10 letters", Ok((20, 640)))]
#[test_case("CF", Err(NoSnellenEquivalent("CF".to_string())))]
#[test_case("HM", Err(NoSnellenEquivalent("HM".to_string())))]
#[test_case("LP", Err(NoSnellenEquivalent("LP".to_string())))]
#[test_case("NLP", Err(NoSnellenEquivalent("NLP".to_string())))]
#[test_case("CF at 20 feet", Ok((20, 73)))] // Schulze-Bonsel et al. (2006)
#[test_case("CF at 2 feet", Ok((20, 738)))] // Schulze-Bonsel et al. (2006)
#[test_case("CF at 30cm", Ok((20, 1500)))] // Schulze-Bonsel et al. (2006)
#[test_case("HM at 20 feet", Ok((20, 196)))] // Schulze-Bonsel et al. (2006)
#[test_case("HM at 2 feet", Ok((20, 1968)))] // Schulze-Bonsel et al. (2006)
#[test_case("HM at 30cm", Ok((20, 4000)))] // Schulze-Bonsel et al. (2006)
#[test_case("CF at 8 feet to 20/400", Err(MultipleValues(format ! (""))))]
fn test_visit_snellen_equivalents(
    text: &str,
    expected: VisualAcuityResult<(u16, u16)>,
) -> VisualAcuityResult<()> {
    let expected = expected.map(|e| Fraction::from(e)).into();
    let parser = Parser::new();
    let visit_notes = BTreeMap::from([("", text)]);
    let parsed = parser.parse_visit(visit_notes.into()).map(|visit| {
        visit
            .into_iter()
            .map(
                |(key, note)| match (key, note.expect("TEST").snellen_equivalent) {
                    (k, OptionResult::Err(MultipleValues(_))) => {
                        (k, OptionResult::Err(MultipleValues(format!(""))))
                    }
                    (k, v) => (k, v),
                },
            )
            .collect::<HashMap<_, _>>()
    })?;
    let actual = parsed.get("").expect("").clone();

    assert_eq!(actual, expected, "{text}");
    Ok(())
}

#[test_case(
    vec ! [("", "20/20 ETDRS 83 letters")],
    Ok(HashMap::from([("", Ok(VAFormat::ETDRS))]))
)]
#[test_case(
    vec ! [("", "CF at 8 feet to 20/400")],
    Ok(HashMap::from([("", Err(MultipleValues(format ! ("[NearTotalLoss, Snellen]"))))]))
)]
fn test_va_format(
    visit_notes: Vec<(&str, &str)>,
    expected: VisualAcuityResult<HashMap<&str, VisualAcuityResult<VAFormat>>>,
) {
    let parser = Parser::new();
    let actual = parser.parse_visit(visit_notes.into()).map(|visit| {
        visit
            .into_iter()
            .map(|(key, note)| (key, note.expect("TEST").va_format))
            .collect()
    });
    let expected = expected.map(|exp| {
        exp.into_iter()
            .map(|(key, note)| (String::from(key), note))
            .collect::<BTreeMap<_, _>>()
    });

    assert_eq!(actual, expected);
}

#[test_case(
    [
        ("Left Eye Distance SC", "20/20"),
        ("Left Eye Distance SC Plus", "-2"),
    ],
    [
        ("Left Eye Distance SC", ("20/20", "-2")),
    ]
)]
#[test_case(
    [
        ("Right Eye Distance CC", "20/20"),
        ("Right Eye Distance CC +/-", "-2"),
    ],
    [
        ("Right Eye Distance CC", ("20/20", "-2")),
    ]
)]
#[test_case(
    [
        ("Both Eyes Distance CC", "20/20"),
        ("Both Eyes Distance CC +/-", "-2"),
        ("Both Eyes Distance SC", "20/20"),
        ("Both Eyes Distance SC Plus", "-1"),
        ("Both Eyes Near CC", "J1+"),
        ("Both Eyes Near CC asdf", "J2"),
        ("Comments", "Forgot glasses today"),
    ],
    [
        ("Both Eyes Distance CC", ("20/20", "-2")),
        ("Both Eyes Distance SC", ("20/20", "-1")),
        ("Both Eyes Near CC", ("J1+", "")),
        ("Both Eyes Near CC asdf", ("J2", "")),
        ("Comments", ("Forgot glasses today", "")),
    ]
)]
#[test_case(
    [
        ("Left Eye Distance SC Plus", "-2")
    ],
    [
        ("Left Eye Distance SC", ("", "-2"))
    ];
    "Map even if the parent key isn't present"
)]
fn test_merge_plus_columns<'a, I, E>(notes: I, expected: E)
where
    I: Into<VisitInput>,
    E: Into<VisitInputMerged>,
{
    let column_merger = ColumnMerger::new(1);
    let actual = column_merger.merge_plus_columns(notes.into());
    assert_eq!(actual, expected.into())
}
