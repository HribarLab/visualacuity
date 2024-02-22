#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, HashMap};
    use lazy_static::lazy_static;
    use test_case::test_case;

    use crate::*;
    use crate::DistanceUnits::*;
    use crate::ParsedItem::*;
    use crate::FixationPreference::*;
    use crate::logmar::LogMarBase;
    use crate::VisualAcuityError::*;
    use crate::structure::Correction::*;
    use crate::structure::DistanceOfMeasurement::*;
    use crate::structure::Laterality::*;
    use crate::structure::*;
    use crate::visit::*;
    use crate::visitinput::*;
    use crate::parser::grammar::*;
    use crate::snellen_equivalent::SnellenEquivalent;

    lazy_static!{
    static ref CHART_NOTES_PARSER: ChartNotesParser = ChartNotesParser::new();
    static ref PLUS_LETTERS_PARSER: PlusLettersParser = PlusLettersParser::new();
    static ref JAEGER_PARSER: JaegerParser = JaegerParser::new();
    static ref SNELLEN_PARSER: SnellenParser = SnellenParser::new();
    static ref DISTANCE_UNITS_PARSER: DistanceUnitsParser = DistanceUnitsParser::new();
    }

    fn parse_notes(notes: &'static str) -> VisualAcuityResult<Vec<ParsedItem>> {
        match CHART_NOTES_PARSER.parse(notes.into()) {
            Ok(p) => Ok(p.into_iter().collect()),
            Err(e) => Err(ParseError(format!("{e:?}: {notes}")))
        }
    }

    #[test]
    fn test_teller() {
        let expected = vec![
            SnellenFraction(format!("20/23")),
            TellerCyCm(format!("38 cy/cm")),
            TellerCard(format!("Card 17")),
        ].into_iter().collect();
        assert_eq!(CHART_NOTES_PARSER.parse("20/23 (38.0 cy/cm) Card 17"), Ok(expected));
        assert_eq!(SnellenFraction(format!("20/23")).snellen_equivalent(), Ok((20, 23).into()), "20/23 fraction");
        assert_eq!(SnellenFraction(format!("20/23")).log_mar_base(), Ok(0.0606978403536117), "20/23 logmar");
        assert_eq!(SnellenFraction(format!("38 cy/cm")).snellen_equivalent(), Ok((20, 23).into()), "38 cy/cm fraction");
        assert_eq!(SnellenFraction(format!("38 cy/cm")).log_mar_base(), Ok(0.0606978403536117), "38 cy/cm logmar");
        assert_eq!(SnellenFraction(format!("Card 17")).snellen_equivalent(), Ok((20, 23).into()), "Card 17 fraction");
        assert_eq!(SnellenFraction(format!("Card 17")).log_mar_base(), Ok(0.0606978403536117), "Card 17 logmar");
    }

    #[test_case(
        "CF @ 30cm",
        Ok(vec![LowVision("CF".to_string(), Centimeters(30.0) )]),
        Ok((20, 1500).into()),
        Ok(1.8750612633917)
    )]
    #[test_case(
        "CF @ 3ft",
        Ok(vec![LowVision("CF".to_string(), Feet(3.0) )]),
        Ok((20, 492).into()),
        Ok(1.390935107103379)
    )]
    fn test_ntlv(
        s: &str,
        expected: VisualAcuityResult<Vec<ParsedItem>>,
        expect_fraction: VisualAcuityResult<Fraction>,
        expect_log_mar: VisualAcuityResult<f64>
    ) -> VisualAcuityResult<()> {
        let expected = expected.map(|exp| exp.into_iter().collect());
        let actual: VisualAcuityResult<_> = CHART_NOTES_PARSER.parse(s)
            .map_err(|e| UnknownError(format!("{e:?}")));
        assert_eq!(actual, expected);
        let item = actual.map(|item| item.into_iter().next().unwrap());
        assert_eq!(item.clone().and_then(|it| it.snellen_equivalent()), expect_fraction);
        assert_eq!(item.clone().and_then(|it| it.log_mar_base()), expect_log_mar);
        Ok(())
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
        SnellenFraction(format!("20/30")),
        PlusLettersItem(1),
    ]) ; "20 / 30 plus one" )]
    #[test_case("20 / 30 - 1", Ok(vec![
        SnellenFraction(format!("20/30")),
        PlusLettersItem(-1),
    ]); "20 / 30 minus one")]
    #[test_case("20 / 30 +3 -1", Ok(vec![
        SnellenFraction(format!("20/30")),
        PlusLettersItem(3),
        PlusLettersItem(-1),
    ]))]
    #[test_case("20/30-1+6", Ok(vec![
        SnellenFraction(format!("20/30")),
        PlusLettersItem(-1),
        PlusLettersItem(6),
    ]))]
    #[test_case("20/987", Ok(vec![
        Text("20/987".to_string()),
    ]))]
    #[test_case("20/321", Ok(vec![
        Text("20/321".to_string()),
    ]); "Make sure 20/321 doesn't get scooped up by 20/32")]
    fn test_fractions_with_plus_letters(chart_note: &str, expected: Result<Vec<ParsedItem>, ()>) {
        let expected = expected.map(|e| e.into_iter().collect());
        assert_eq!(CHART_NOTES_PARSER.parse(chart_note).map_err(|_| ()), expected);
    }

    #[test_case("J1", Ok(Jaeger("J1".to_string())))]
    #[test_case("J1+", Ok(Jaeger("J1+".to_string())); "J1plus")]
    #[test_case("J29", Ok(Jaeger("J29".to_string())))]
    #[test_case("J0", Ok(Jaeger("J1+".to_string())))]
    fn test_jaeger(chart_note: &str, expected: VisualAcuityResult<ParsedItem>) {
        let actual = JAEGER_PARSER.parse(chart_note).map_err(|e| UnknownError(format!("{e:?}")));
        assert_eq!(actual, expected, "{chart_note}");
    }

    #[test_case("CF", Ok(vec![LowVision("CF".to_string(), NotProvided )]))]
    #[test_case("HM", Ok(vec![LowVision("HM".to_string(), NotProvided )]))]
    #[test_case("LP", Ok(vec![LowVision("LP".to_string(), NotProvided )]))]
    #[test_case("NLP", Ok(vec![LowVision("NLP".to_string(), NotProvided )]))]
    #[test_case("BTL", Ok(vec![LowVision("LP".to_string(), NotProvided )]))]
    #[test_case("blink to light", Ok(vec![LowVision("LP".to_string(), NotProvided )]))]
    #[test_case("NI", Ok(vec![PinHoleEffectItem(PinHoleEffect::NI)]))]
    #[test_case("CF at 1.5ft", Ok(vec![LowVision("CF".to_string(), Feet(1.5) )]))]
    #[test_case("CF 2'", Ok(vec![LowVision("CF".to_string(), Feet(2.0) )]))]
    #[test_case("CF@3'", Ok(vec![LowVision("CF".to_string(), Feet(3.0) )]))]
    #[test_case("CF at 3'", Ok(vec![LowVision("CF".to_string(), Feet(3.0) )]))]
    #[test_case("CF @ 3 feet", Ok(vec![LowVision("CF".to_string(), Feet(3.0) )]))]
    #[test_case("CF @ face", Ok(vec![LowVision("CF".to_string(), DistanceUnits::Unhandled("CF @ face".to_string()) )]))]
    #[test_case("CF @ 2M", Ok(vec![LowVision("CF".to_string(), Meters(2.0) )]))]
    #[test_case("CF @ 0.3 meters", Ok(vec![LowVision("CF".to_string(), Meters(0.3) )]))]
    #[test_case("CF @ 30 cm", Ok(vec![LowVision("CF".to_string(), Centimeters(30.0) )]))]
    #[test_case("No BTL", Ok(vec![LowVision("NLP".to_string(), NotProvided )]))]
    #[test_case("CSM", Ok(vec![BinocularFixation(CSM)]))]
    fn test_alternative_visual_acuity(chart_note: &str, expected: Result<Vec<ParsedItem>, ()>) {
        let expected = expected.map(|e| e.into_iter().collect());
        assert_eq!(CHART_NOTES_PARSER.parse(chart_note).map_err(|_| ()), expected, "{chart_note:?}");
    }

    #[test]
    fn test_distance_conversion() {
        assert_eq!(Centimeters(30.0).to_feet(), Ok(0.984252));
        assert_eq!(Meters(0.3).to_feet(), Ok(0.9842519999999999));
    }


    #[test_case("20/20 +1 -3", Ok(vec![
        SnellenFraction(format!("20/20")),
        PlusLettersItem(1),
        PlusLettersItem(-3),
    ]))]
    #[test_case("ok", Ok(vec![
        Text("ok".to_string()),
    ]))]
    #[test_case("ok 20/20 +1 -3", Ok(vec![
        Text("ok".to_string()),
        SnellenFraction(format!("20/20")),
        PlusLettersItem(1),
        PlusLettersItem(-3),
    ]))]
    #[test_case("20/20 +1 -3 asdf", Ok(vec![
        SnellenFraction(format!("20/20")),
        PlusLettersItem(1),
        PlusLettersItem(-3),
        Text("asdf".to_string()),
    ]))]
    #[test_case("20/20 +1 -3 asdf qwerty", Ok(vec![
        SnellenFraction(format!("20/20")),
        PlusLettersItem(1),
        PlusLettersItem(-3),
        Text("asdf qwerty".to_string()),
    ]))]
    #[test_case("12/20 +1 -3", Ok(vec![
        Text("12/20".to_string()),
        PlusLettersItem(1),
        PlusLettersItem(-3),
    ]))]
    fn test_plain_text_notes(chart_note: &'static str, expected: Result<Vec<ParsedItem>, ()>) {
        assert_eq!(parse_notes(chart_note).map_err(|_| ()), expected);
    }

    #[test_case("20/20-1", Ok(vec![
        SnellenFraction(format!("20/20")),
        PlusLettersItem(-1),
    ]))]
    #[test_case("20/125", Ok(vec![
        SnellenFraction(format!("20/125")),
    ]))]
    #[test_case("20/23 (38.0 cy/cm) Card 17", Ok(vec![
        SnellenFraction(format!("20/23")),
        TellerCyCm(format!("38 cy/cm")),
        TellerCard(format!("Card 17")),
    ]))]
    #[test_case("20/130 (6.5 cy/cm) Card 12", Ok(vec![
        SnellenFraction(format!("20/130")),
        TellerCyCm(format!("6.5 cy/cm")),
        TellerCard(format!("Card 12")),
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
        Text("good".to_string()),
        BinocularFixation(FixAndFollow),
    ]))]
    fn test_fix_and_follow(chart_note: &'static str, expected: Result<Vec<ParsedItem>, ()>) {
        assert_eq!(parse_notes(chart_note).map_err(|_| ()), expected, "{chart_note}");
    }

    #[test_case("CSM Pref", Ok(vec![BinocularFixation(CSM), BinocularFixation(Prefers)]))]
    #[test_case("j1", Ok(vec![Jaeger("J1".to_string())]))]
    #[test_case("j30", Ok(vec![Text("j30".to_string())]))]
    #[test_case("79 letters", Ok(vec![ETDRS("79 letters".to_string())]))]
    #[test_case("81ltrs", Ok(vec![ETDRS("81 letters".to_string())]))]
    // #[test_case("20/13 ETDRS (95 letters)", Ok(vec![ETDRS { letters: 95 }]))]
    #[test_case("20/20 ETDRS (83 letters)", Ok(vec![SnellenFraction(format!("20/20")), ETDRS("83 letters".to_string())]))]
    fn test_various(chart_note: &'static str, expected: Result<Vec<ParsedItem>, ()>) {
        assert_eq!(parse_notes(chart_note).map_err(|_| ()), expected);
    }

    #[test_case("20/20 +1", "", (SnellenFraction(format!("20/20")), vec![1]))]
    #[test_case("J4 +2", "", (Jaeger("J4".to_string()), vec![2]))]
    fn test_base_and_plus_letters(
        notes: &str,
        plus: &str,
        expected: (ParsedItem, Vec<i32>)
    ) -> Result<(), anyhow::Error> {
        let parser = Parser::new();
        let parsed_notes = parser.parse_text(notes);
        let parsed_plus = parser.parse_text(plus);
        let combined: ParsedItemCollection = [parsed_notes, parsed_plus].into_iter().flatten().collect();
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
        let parser = Parser::new();
        let actual = parser.parse_text(notes);
        let expected = expected.map(|e| e.into_iter().collect())?;
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test_case("20/20",  Ok(Some(0.0)))]
    #[test_case("20/15",  Ok(Some(-0.12493873660829993)))]
    #[test_case("J2",  Ok(Some(0.09691001300805639)))]
    #[test_case("J1",  Ok(Some(0.0)))]
    #[test_case("CF",  Err(NotImplemented))]
    #[test_case("HM",  Err(NotImplemented))]
    #[test_case("LP",  Err(NotImplemented))]
    #[test_case("NLP",  Err(NotImplemented))]
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
        let parser = Parser::new();
        let visit_notes = [("fieldname", notes)].into();
        let parsed_notes = parser.parse_visit(visit_notes);
        let base_item = parsed_notes?.into_iter().map(|(_, v)| v).next().expect("");
        let approx = |x| Some(format!("{:.8}", x?));
        assert_eq!(base_item.log_mar_base.map(approx), expected.map(approx), "\"{notes}\"");
        Ok(())
    }

    #[test_case(
        [
            ("Both Eyes Distance CC", "20/20"),
            ("Both Eyes Distance CC +/-", "-2"),
        ],
        Ok([
            (("Both Eyes Distance CC".to_string()), VisitNote {
                text: "20/20".to_string(),
                text_plus: "-2".to_string(),
                laterality: Laterality::OU,
                distance_of_measurement: DistanceOfMeasurement::Distance,
                correction: Correction::CC,
                pinhole: PinHole::Unknown,
                method: Method::Snellen,
                extracted_value: format!("20/20"),
                plus_letters: vec![-2],
                snellen_equivalent: Ok(Some((20, 20).into())),
                log_mar_base: Ok(Some(0.0)),
                log_mar_base_plus_letters: Ok(Some(0.03230333766935213)),
            }),
        ])
    )]
    #[test_case(
        [
            ("Visual Acuity Right Eye Distance CC", "20/100-1+2 ETDRS  (sc eccentric fixation)"),
        ],
        Ok([
            ("Visual Acuity Right Eye Distance CC".to_string(), VisitNote {
                text: "20/100-1+2 ETDRS  (sc eccentric fixation)".to_string(),
                text_plus: "".to_string(),
                laterality: Laterality::OD,
                distance_of_measurement: DistanceOfMeasurement::Distance,
                correction: Correction::Error(MultipleValues(format!("[CC, SC]"))),
                pinhole: PinHole::Unknown,
                method: Method::Snellen,
                extracted_value: format!("20/100"),
                plus_letters: vec![-1, 2],
                snellen_equivalent: Ok(Some((20, 100).into())),
                log_mar_base: Ok(Some(0.6989700043360187)),
                log_mar_base_plus_letters: Ok(Some(0.6828183355013427)),
            })
        ])
        ; "Handling ambiguous Methods"
    )]
    #[test_case(
        [
            ("Visual Acuity Right Eye Distance CC", "20/20 J5"),
        ],
        Ok([
            ("Visual Acuity Right Eye Distance CC".to_string(), VisitNote {
                text: "20/20 J5".to_string(),
                text_plus: "".to_string(),
                laterality: Laterality::OD,
                distance_of_measurement: DistanceOfMeasurement::Distance,
                correction: Correction::CC,
                pinhole: PinHole::Unknown,
                method: Method::Error(MultipleValues(format!("[Snellen, Jaeger]"))),
                extracted_value: format!("Error"),
                plus_letters: vec![],
                snellen_equivalent: Err(MultipleValues(format!("[SnellenFraction(\"20/20\"), Jaeger(\"J5\")]"))),
                log_mar_base: Err(MultipleValues(format!("[SnellenFraction(\"20/20\"), Jaeger(\"J5\")]"))),
                log_mar_base_plus_letters: Err(MultipleValues(format!("[SnellenFraction(\"20/20\"), Jaeger(\"J5\")]"))),
            })
        ])
        ; "Handling ambiguous Correction values"
    )]
    fn test_visit<'a, I, E>(
        visit_notes: I,
        expected: VisualAcuityResult<E>
    ) where I: Into<VisitInput>, E: Into<BTreeMap<String, VisitNote>> {
        let expected = expected.map(|v| Visit(v.into()));
        let parser = Parser::new();
        let visit_notes = visit_notes.into();
        let actual = parser.parse_visit(visit_notes.clone());
        assert_eq!(actual, expected, "{visit_notes:?}");
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
        Ok([("", "CF @ 2.0 feet")])
    )]
    #[test_case(
        [("", "CF at 8 feet to 20/400")],
        Ok([("", "Error")])
    )]
    fn test_extracted_value<T: Into<VisitInput>>(visit_notes: T, expected: VisualAcuityResult<T>) {
        let parser = Parser::new();
        let visit_notes = visit_notes.into();
        let expected = expected.map(|exp| exp.into());
        let actual = parser.parse_visit(visit_notes.clone())
            .map(|visit| {
                visit.into_iter()
                    .map(|(key, note)| (key, note.extracted_value))
                    .into()
            });
        assert_eq!(actual, expected, "{visit_notes:?}")
    }

    #[test_case("20/20",  Ok(Some((20, 20))))]
    #[test_case("20/15",  Ok(Some((20, 15))))]
    #[test_case("J1",  Ok(Some((20, 20))))]
    #[test_case("J2",  Ok(Some((20, 25))))]
    #[test_case("70 letters",  Ok(Some((20, 40))))]
    #[test_case("58 letters",  Ok(Some((20, 63))))]
    #[test_case("10 letters",  Ok(Some((20, 640))))]
    #[test_case("CF", Err(NoSnellenEquivalent("CF".to_string())))]
    #[test_case("HM", Err(NoSnellenEquivalent("HM".to_string())))]
    #[test_case("LP", Err(NoSnellenEquivalent("LP".to_string())))]
    #[test_case("NLP", Err(NoSnellenEquivalent("NLP".to_string())))]

    #[test_case("CF at 20 feet",  Ok(Some((20, 73))))]   // Schulze-Bonsel et al. (2006)
    #[test_case("CF at 2 feet",  Ok(Some((20, 738))))]   // Schulze-Bonsel et al. (2006)
    #[test_case("CF at 30cm",  Ok(Some((20, 1500))))]    // Schulze-Bonsel et al. (2006)
    #[test_case("HM at 20 feet",  Ok(Some((20, 196))))]  // Schulze-Bonsel et al. (2006)
    #[test_case("HM at 2 feet",  Ok(Some((20, 1968))))]  // Schulze-Bonsel et al. (2006)
    #[test_case("HM at 30cm",  Ok(Some((20, 4000))))]    // Schulze-Bonsel et al. (2006)

    #[test_case("CF at 8 feet to 20/400", Err(MultipleValues(format!(""))))]
    fn test_visit_snellen_equivalents(
        text: &str,
        expected: VisualAcuityResult<Option<(u16, u16)>>
    ) -> Result<(), anyhow::Error> {
        let expected = expected.map(|e| e.map(Into::into));
        let parser = Parser::new();
        let visit_notes = BTreeMap::from([("", text)]);
        let parsed = parser.parse_visit(visit_notes.into())
            .map(|visit| {
                visit.into_iter()
                    .map(|(key, note)| match (key, note.snellen_equivalent) {
                        (k, Err(MultipleValues(_))) => (k,  Err(MultipleValues(format!("")))),
                        (k, v) => (k, v)
                    })
                    .collect::<HashMap<_, _>>()
            })?;
        let actual = parsed.get("").expect("").clone();

        assert_eq!(actual, expected, "{text}");
        Ok(())
    }

    #[test_case(
        vec![("", "20/20 ETDRS 83 letters")],
        Ok(HashMap::from([("", Method::ETDRS)]))
    )]
    #[test_case(
        vec![("", "CF at 8 feet to 20/400")],
        Ok(HashMap::from([("", Method::Error(MultipleValues(format!("[LowVision, Snellen]"))))]))
    )]
    fn test_method(
        visit_notes: Vec<(&str, &str)>,
        expected: VisualAcuityResult<HashMap<&str, Method>>
    ) {
        let parser = Parser::new();
        let actual = parser.parse_visit(visit_notes.into())
            .map(|visit| {
                visit.into_iter()
                    .map(|(key, note)| (key, note.method))
                    .collect()
            });
        let expected = expected.map(|exp| exp.into_iter()
            .map(|(key, note)| (String::from(key), note))
            .collect::<BTreeMap<_, _>>()
        );

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
        where I: Into<VisitInput>, E: Into<VisitInputMerged>
    {
        let column_merger = ColumnMerger::new(1);
        let actual = column_merger.merge_plus_columns(notes.into());
        assert_eq!(actual, expected.into())
    }
}
