#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use lazy_static::lazy_static;
    use crate::JaegerRow::*;
    use crate::SnellenRow::*;
    use crate::ParsedItem::*;
    use crate::LowVisionMethod::*;
    use crate::parser::*;
    use crate::{Parser, ParsedItem, ParsedItemCollection, VisualAcuityResult, PinHoleEffect,
                VisitNote, Laterality, DistanceOfMeasurement, Correction};
    use crate::FixationPreference::*;
    use crate::VisualAcuityError::{*};

    use test_case::test_case;
    use crate::structure::Correction::*;
    use crate::structure::DistanceOfMeasurement::*;
    use crate::structure::Laterality::*;
    use crate::structure::{Method, PinHole};
    use crate::visit::{merge_plus_columns, Visit};

    lazy_static!{
    static ref PARSER: Parser = Parser::new();
    static ref CHART_NOTES_PARSER: grammar::ChartNotesParser = grammar::ChartNotesParser::new();
    static ref PLUS_LETTERS_PARSER: grammar::PlusLettersParser = grammar::PlusLettersParser::new();
    static ref JAEGER_PARSER: grammar::JaegerParser = grammar::JaegerParser::new();
    static ref AT_DISTANCE_PARSER: grammar::AtDistanceParser = grammar::AtDistanceParser::new();
    }

    fn parse_notes(notes: &'static str) -> VisualAcuityResult<Vec<ParsedItem>> {
        match CHART_NOTES_PARSER.parse(notes.into()) {
            Ok(p) => Ok(p.into_iter().collect()),
            Err(e) => Err(ParseError(format!("{e:?}: {notes}")))
        }
    }

    #[test_case("+6", Ok(PlusLettersItem(6)); "plus 6 without a space")]
    #[test_case("+ 6", Ok(PlusLettersItem(6)); "plus 6 with a space")]
    #[test_case("-5", Ok(PlusLettersItem(-5)))]
    #[test_case("+0", Err(()))]
    #[test_case("+", Err(()); "just a plus sign")]
    #[test_case("-", Err(()); "just a minus sign")]
    #[test_case("-7", Err(()))]
    fn test_plus_letters(chart_note: &str, expected: Result<ParsedItem, ()>) {
        assert_eq!(PLUS_LETTERS_PARSER.parse(chart_note).map_err(|_| ()), expected);
    }

    #[test_case("20 / 30 + 1", Ok(vec![
        Snellen(S30),
        PlusLettersItem(1),
    ]) ; "20 / 30 plus one" )]
    #[test_case("20 / 30 - 1", Ok(vec![
        Snellen(S30),
        PlusLettersItem(-1),
    ]); "20 / 30 minus one")]
    #[test_case("20 / 30 +3 -1", Ok(vec![
        Snellen(S30),
        PlusLettersItem(3),
        PlusLettersItem(-1),
    ]))]
    #[test_case("20/30-1+6", Ok(vec![
        Snellen(S30),
        PlusLettersItem(-1),
        PlusLettersItem(6),
    ]))]
    #[test_case("20/987", Ok(vec![
        Text("20/987"),
    ]))]
    fn test_fractions_with_plus_letters(chart_note: &str, expected: Result<Vec<ParsedItem>, ()>) {
        assert_eq!(CHART_NOTES_PARSER.parse(chart_note).map_err(|_| ()), expected.map(|e| ParsedItemCollection(e)));
    }

    #[test_case("J1", Ok(Jaeger(J1)))]
    #[test_case("J1+", Ok(Jaeger(J1PLUS)))]
    #[test_case("J29", Ok(Jaeger(J29)))]
    fn test_jaeger(chart_note: &str, expected: Result<ParsedItem, ()>) {
        assert_eq!(JAEGER_PARSER.parse(chart_note).map_err(|_| ()), expected);
    }

    #[test_case("CF", Ok(vec![LowVision { method: CountingFingers, distance: None }]))]
    #[test_case("HM", Ok(vec![LowVision { method: HandMovement, distance: None }]))]
    #[test_case("LP", Ok(vec![LowVision { method: LightPerception, distance: None }]))]
    #[test_case("NLP", Ok(vec![LowVision { method: NoLightPerception, distance: None }]))]
    #[test_case("BTL", Ok(vec![LowVision { method: LightPerception, distance: None }]))]
    #[test_case("blink to light", Ok(vec![LowVision{method: LightPerception, distance: None }]))]
    #[test_case("NI", Ok(vec![PinHoleEffectItem(PinHoleEffect::NI)]))]
    #[test_case("CF at 1.5ft", Ok(vec![LowVision { method: CountingFingers, distance: Some(Distance) }]))]
    #[test_case("CF 2'", Ok(vec![LowVision { method: CountingFingers, distance: Some(Distance) }]))]
    #[test_case("CF@3'", Ok(vec![LowVision { method: CountingFingers, distance: Some(Distance) }]))]
    // #[test_case("CF at 3'", Ok(vec![LowVision { method: CountingFingers, distance: Some(Feet(3.0)) }]))]
    // #[test_case("CF @ 3 feet", Ok(vec![LowVision { method: CountingFingers, distance: Some(Feet(3.0)) }]))]
    // #[test_case("CF @ face", Ok(vec![LowVision { method: CountingFingers, distance: Some(Face) }]))]
    // #[test_case("No BTL", Ok(vec![LowVision { method: NoLightPerception, distance: None }]))]
    #[test_case("CSM", Ok(vec![BinocularFixation(CSM)]))]
    fn test_alternative_visual_acuity(chart_note: &str, expected: Result<Vec<ParsedItem>, ()>) {
        assert_eq!(CHART_NOTES_PARSER.parse(chart_note).map_err(|_| ()), expected.map(|e| ParsedItemCollection(e)));
    }

    #[test_case("20/20 +1 -3", Ok(vec![
        Snellen(S20),
        PlusLettersItem(1),
        PlusLettersItem(-3),
    ]))]
    #[test_case("ok", Ok(vec![
        Text("ok"),
    ]))]
    #[test_case("ok 20/20 +1 -3", Ok(vec![
        Text("ok"),
        Snellen(S20),
        PlusLettersItem(1),
        PlusLettersItem(-3),
    ]))]
    #[test_case("20/20 +1 -3 asdf", Ok(vec![
        Snellen(S20),
        PlusLettersItem(1),
        PlusLettersItem(-3),
        Text("asdf"),
    ]))]
    #[test_case("20/20 +1 -3 asdf qwerty", Ok(vec![
        Snellen(S20),
        PlusLettersItem(1),
        PlusLettersItem(-3),
        Text("asdf qwerty"),
    ]))]
    #[test_case("12/20 +1 -3", Ok(vec![
        Text("12/20"),
        PlusLettersItem(1),
        PlusLettersItem(-3),
    ]))]
    fn test_plain_text_notes(chart_note: &'static str, expected: Result<Vec<ParsedItem>, ()>) {
        assert_eq!(parse_notes(chart_note).map_err(|_| ()), expected);
    }

    #[test_case("20/20-1", Ok(vec![
        Snellen(S20),
        PlusLettersItem(-1),
    ]))]
    #[test_case("20/125", Ok(vec![
        Snellen(S125),
    ]))]
    #[test_case("20/23 (38.0 cy/cm) Card 17", Ok(vec![
        Teller { row: 23, card: 17 },
    ]))]
    #[test_case("20/130 (6.5 cy/cm) Card 12", Ok(vec![
        Teller { row: 130, card: 12 },
    ]))]
    fn test_whole_thing(chart_note: &'static str, expected: Result<Vec<ParsedItem>, ()>) {
        assert_eq!(parse_notes(chart_note).map_err(|_| ()), expected);
    }

    #[test_case("fix and follow", Ok(vec![
        BinocularFixation(FixAndFollow)
    ]))]
    #[test_case("fix & follow", Ok(vec![
        BinocularFixation(FixAndFollow)
    ]))]
    #[test_case("FF", Ok(vec![
        BinocularFixation(FixAndFollow),
    ]))]
    #[test_case("F + F", Ok(vec![
        BinocularFixation(FixAndFollow),
    ]); "F + F with plus sign and spaces")]
    #[test_case("F+F", Ok(vec![
        BinocularFixation(FixAndFollow),
    ]); "F + F with plus sign no spaces")]
    #[test_case("F&F", Ok(vec![
        BinocularFixation(FixAndFollow),
    ]); "F + F with ampersand no spaces")]
    #[test_case("f/f", Ok(vec![
        BinocularFixation(FixAndFollow),
    ]); "F + F with slash no spaces")]
    #[test_case("no fix & follow", Ok(vec![
        BinocularFixation(NoFixAndFollow),
    ]))]
    #[test_case("Fix, No Follow", Ok(vec![
        BinocularFixation(FixNoFollow),
    ]))]
    #[test_case("CSM, good f+f", Ok(vec![
        BinocularFixation(CSM),
        Text("good"),
        BinocularFixation(FixAndFollow),
    ]))]
    fn test_fix_and_follow(chart_note: &'static str, expected: Result<Vec<ParsedItem>, ()>) {
        assert_eq!(parse_notes(chart_note).map_err(|_| ()), expected);
    }

    #[test_case("CSM Pref", Ok(vec![BinocularFixation(CSM), BinocularFixation(Prefers)]))]
    #[test_case("j1", Ok(vec![Jaeger(J1)]))]
    #[test_case("79 letters", Ok(vec![ETDRS { letters: 79 }]))]
    #[test_case("81ltrs", Ok(vec![ETDRS { letters: 81 }]))]
    // #[test_case("20/13 ETDRS (95 letters)", Ok(vec![ETDRS { letters: 95 }]))]
    #[test_case("20/20 ETDRS (83 letters)", Ok(vec![Snellen(S20), ETDRS { letters: 83 }]))]
    fn test_various(chart_note: &'static str, expected: Result<Vec<ParsedItem>, ()>) {
        assert_eq!(parse_notes(chart_note).map_err(|_| ()), expected);
    }

    #[test_case("20/20 +1", "", (Snellen(S20), vec![1]))]
    #[test_case("J4 +2", "", (Jaeger(J4), vec![2]))]
    fn test_base_and_plus_letters(
        notes: &str,
        plus: &str,
        expected: (ParsedItem, Vec<i32>)
    ) -> Result<(), anyhow::Error> {
        let parsed_notes = PARSER.parse_notes(notes);
        let parsed_plus = PARSER.parse_notes(plus);
        let combined = ParsedItemCollection(parsed_notes.into_iter().chain(parsed_plus).collect());
        let base_item = combined.base_acuity()?;
        let plus_letters = combined.plus_letters();
        assert_eq!((base_item, plus_letters), expected);
        Ok(())
    }

    #[test_case("Both Eyes Distance CC", Ok(vec![
        LateralityItem(OU),
        DistanceItem(Distance),
        CorrectionItem(CC),
    ]))]
    fn test_visit_details(
        notes: &str,
        expected: VisualAcuityResult<Vec<ParsedItem>>
    ) -> Result<(), anyhow::Error> {
        let actual = PARSER.parse_notes(notes);
        assert_eq!(Ok(actual), expected.map(|e| ParsedItemCollection(e)));
        Ok(())
    }

    #[test_case("20/20",  Ok(Some(0.0)))]
    #[test_case("20/15",  Ok(Some(-0.12493873660829993)))]
    #[test_case("J2",  Ok(Some(0.09691001300805639)))]
    #[test_case("J1",  Ok(Some(0.0)))]
    #[test_case("CF",  Ok(Some(1.9)))]
    #[test_case("HM",  Ok(Some(2.3)))]
    #[test_case("LP",  Ok(Some(2.7)))]
    #[test_case("NLP",  Ok(Some(3.0)))]
    #[test_case("5 letters",  Ok(Some(1.6)))]
    #[test_case("10 letters",  Ok(Some(1.5)))]
    #[test_case("15 letters",  Ok(Some(1.4)))]
    #[test_case("20 letters",  Ok(Some(1.3)))]
    #[test_case("25 letters",  Ok(Some(1.2)))]
    #[test_case("30 letters",  Ok(Some(1.1)))]
    #[test_case("35 letters",  Ok(Some(1.0)))]
    #[test_case("40 letters",  Ok(Some(0.9)))]
    #[test_case("45 letters",  Ok(Some(0.8)))]
    #[test_case("50 letters",  Ok(Some(0.7)))]
    #[test_case("55 letters",  Ok(Some(0.6)))]
    #[test_case("60 letters",  Ok(Some(0.5)))]
    #[test_case("65 letters",  Ok(Some(0.4)))]
    #[test_case("70 letters",  Ok(Some(0.3)))]
    #[test_case("75 letters",  Ok(Some(0.2)))]
    #[test_case("80 letters",  Ok(Some(0.1)))]
    #[test_case("85 letters",  Ok(Some(0.0)))]
    #[test_case("90 letters",  Ok(Some(-0.1)))]
    #[test_case("95 letters",  Ok(Some(-0.2)))]
    #[test_case("58 letters",  Ok(Some(0.5)); "TODO: treat n%5 as plus_letters?")]
    fn test_log_mar_base(
        notes: &str,
        expected: VisualAcuityResult<Option<f64>>
    ) -> Result<(), anyhow::Error> {
        let visit_notes = HashMap::from([("fieldname", notes)]);
        let parsed_notes = PARSER.parse_visit(visit_notes);
        let base_item = parsed_notes?.into_iter().map(|(_, v)| v).next().expect("");
        assert_eq!(base_item.log_mar_base, expected);
        Ok(())
    }

    #[test_case(
        vec![
            ("Both Eyes Distance CC", "20/20"),
            ("Both Eyes Distance CC +/-", "-2"),
        ],
        Ok(Visit(HashMap::from([
            (("Both Eyes Distance CC"), VisitNote {
                text: "20/20",
                text_plus: "-2",
                laterality: Laterality::OU,
                distance_of_measurement: DistanceOfMeasurement::Distance,
                correction: Correction::CC,
                pinhole: PinHole::Unknown,
                method: Method::Snellen,
                extracted_value: format!("20/20"),
                plus_letters: vec![-2],
                snellen_equivalent: Ok(Some((20, 20))),
                log_mar_base: Ok(Some(0.0)),
                log_mar_base_plus_letters: Ok(Some(0.03230333766935214)),
            }),
        ])))
    )]
    #[test_case(
        vec![
            ("Visual Acuity Right Eye Distance CC", "20/100-1+2 ETDRS  (sc eccentric fixation)"),
        ],
        Ok(Visit(HashMap::from([
            ("Visual Acuity Right Eye Distance CC", VisitNote {
                text: "20/100-1+2 ETDRS  (sc eccentric fixation)",
                text_plus: "",
                laterality: Laterality::OD,
                distance_of_measurement: DistanceOfMeasurement::Distance,
                correction: Correction::Error(MultipleValues(format!("[CC, SC]"))),
                pinhole: PinHole::Unknown,
                method: Method::Snellen,
                extracted_value: format!("20/100"),
                plus_letters: vec![-1, 2],
                snellen_equivalent: Ok(Some((20, 100))),
                log_mar_base: Ok(Some(0.6989700043360187)),
                log_mar_base_plus_letters: Ok(Some(0.6828183355013427)),
            })
        ])))
        ; "Handling ambiguous Methods"
    )]
    #[test_case(
        vec![
            ("Visual Acuity Right Eye Distance CC", "20/20 J5"),
        ],
        Ok(Visit(HashMap::from([
            ("Visual Acuity Right Eye Distance CC", VisitNote {
                text: "20/20 J5",
                text_plus: "",
                laterality: Laterality::OD,
                distance_of_measurement: DistanceOfMeasurement::Distance,
                correction: Correction::CC,
                pinhole: PinHole::Unknown,
                method: Method::Error(MultipleValues(format!("[Snellen, Jaeger]"))),
                extracted_value: format!("Error"),
                plus_letters: vec![],
                snellen_equivalent: Err(MultipleValues(format!("[Snellen(S20), Jaeger(J5)]"))),
                log_mar_base: Err(MultipleValues(format!("[Snellen(S20), Jaeger(J5)]"))),
                log_mar_base_plus_letters: Err(MultipleValues(format!("[Snellen(S20), Jaeger(J5)]"))),
            })
        ])))
        ; "Handling ambiguous Correction values"
    )]
    fn test_visit(
        visit_notes: Vec<(&str, &str)>,
        expected: VisualAcuityResult<Visit>
    ) {
        let visit_notes = visit_notes.into_iter().collect();
        let actual = PARSER.parse_visit(visit_notes);
        assert_eq!(actual, expected);
    }

    #[test_case(
        vec![("", "NI")],
        Ok(HashMap::from([("", format!("NI"))]))
    )]
    #[test_case(
        vec![("", "NT")],
        Ok(HashMap::from([("", format!("NT"))]))
    )]
    #[test_case(
        vec![("", "unable")],
        Ok(HashMap::from([("", format!("Unable"))]))
    )]
    #[test_case(
        vec![("", "prosthesis")],
        Ok(HashMap::from([("", format!("Prosthesis"))]))
    )]
    #[test_case(
        vec![("", "CF 2'")],
        Ok(HashMap::from([("", format!("CountingFingers"))]))
    )]
    #[test_case(
        vec![("", "CF at 8 feet to 20/400")],
        Ok(HashMap::from([("", format!("Error"))]))
    )]
    fn test_extracted_value(
        visit_notes: Vec<(&str, &str)>,
        expected: VisualAcuityResult<HashMap<&str, String>>
    ) {
        let visit_notes = visit_notes.into_iter().collect();
        let actual = PARSER.parse_visit(visit_notes)
            .map(|visit| {
                visit.into_iter()
                    .map(|(key, note)| (key, note.extracted_value))
                    .collect()
            });

        assert_eq!(actual, expected);
    }

    #[test_case("20/20",  Ok(Some((20, 20))))]
    #[test_case("20/15",  Ok(Some((20, 15))))]
    #[test_case("J1",  Ok(Some((20, 20))))]
    #[test_case("J2",  Ok(Some((20, 25))))]
    #[test_case("70 letters",  Ok(Some((20, 40))))]
    #[test_case("58 letters",  Ok(Some((20, 63))))]
    #[test_case("10 letters",  Ok(Some((20, 640))))]
    #[test_case("CF",  Ok(Some((20, 1600))))]
    #[test_case("HM",  Ok(Some((20, 4000))))]
    #[test_case("LP",  Ok(Some((20, 10000))))]
    #[test_case("NLP",  Ok(Some((20, 20000))))]
    #[test_case("CF at 8 feet to 20/400", Err(MultipleValues(format!(""))))]
    fn test_visit_snellen_equivalents(
        text: &str,
        expected: VisualAcuityResult<Option<(u16, u16)>>
    ) -> Result<(), anyhow::Error> {
        let visit_notes = HashMap::from([("", text)]);
        let parsed = PARSER.parse_visit(visit_notes)
            .map(|visit| {
                visit.into_iter()
                    .map(|(key, note)| match (key, note.snellen_equivalent) {
                        (_, Err(MultipleValues(_))) => (key,  Err(MultipleValues(format!("")))),
                        (k, v) => (k, v)
                    })
                    .collect::<HashMap<_, _>>()
            })?;
        let actual = parsed.get("").expect("").clone();

        assert_eq!(actual, expected);
        Ok(())
    }

    #[test_case(
        vec![("", "20/20 ETDRS 83 letters")],
        Ok(HashMap::from([("", Method::ETDRS)]))
    )]
    fn test_method(
        visit_notes: Vec<(&str, &str)>,
        expected: VisualAcuityResult<HashMap<&str, Method>>
    ) {
        let visit_notes = visit_notes.into_iter().collect();
        let actual = PARSER.parse_visit(visit_notes)
            .map(|visit| {
                visit.into_iter()
                    .map(|(key, note)| (key, note.method))
                    .collect()
            });

        assert_eq!(actual, expected);
    }

    #[test_case(
        HashMap::from([
            ("Left Eye Distance SC", vec!["20/20"]),
            ("Left Eye Distance SC Plus", vec!["-2"]),
        ]),
        HashMap::from([
            ("Left Eye Distance SC", vec!["20/20", "-2"]),
        ])
    )]
    #[test_case(
        HashMap::from([
            ("Right Eye Distance CC", vec!["20/20"]),
            ("Right Eye Distance CC +/-", vec!["-2"]),
        ]),
        HashMap::from([
            ("Right Eye Distance CC", vec!["20/20", "-2"]),
        ])
    )]
    #[test_case(
        HashMap::from([
            ("Both Eyes Distance CC", vec!["20/20"]),
            ("Both Eyes Distance CC +/-", vec!["-2"]),
            ("Both Eyes Distance SC", vec!["20/20"]),
            ("Both Eyes Distance SC Plus", vec!["-1"]),
            ("Both Eyes Near CC", vec!["J1+"]),
            ("Both Eyes Near CC asdf", vec!["J2"]),
            ("Comments", vec!["Forgot glasses today"]),
        ]),
        HashMap::from([
            ("Both Eyes Distance CC", vec!["20/20", "-2"]),
            ("Both Eyes Distance SC", vec!["20/20", "-1"]),
            ("Both Eyes Near CC", vec!["J1+"]),
            ("Both Eyes Near CC asdf", vec!["J2"]),
            ("Comments", vec!["Forgot glasses today"]),
        ])
    )]
    #[test_case(
        HashMap::from([
            ("Left Eye Distance SC Plus", vec!["-2"])
        ]),
        HashMap::from([
            ("Left Eye Distance SC", vec!["-2"])
        ]);
        "Map even if the parent key isn't present"
    )]
    fn test_merge_plus_columns(
        notes: HashMap<&str, Vec<&str>>,
        expected: HashMap<&str, Vec<&str>>
    ) {
        let actual = merge_plus_columns(notes);
        assert_eq!(actual, expected)
    }
}
