#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::fmt::Debug;
    use test_case::test_case;

    use crate::visit::Correction::*;
    use crate::visit::DistanceOfMeasurement::*;
    use crate::visit::Laterality::*;
    use crate::visit::*;
    use crate::*;

    type R<T> = VisualAcuityResult<T>;

    #[test]
    fn test_base_and_plus_letters() -> R<()> {
        let notes = "CSM";

        let parsed_notes = Parser::new().parse_text(notes).content;
        let sifted = &SiftedParsedItems::sift(parsed_notes);

        assert_eq!(vec![VisualResponse(s!("CSM"))], sifted.acuities);
        assert_eq!(
            OptionResult::Some(VisualResponse(s!("CSM"))),
            sifted.base_acuity_()
        );
        Ok(())
    }

    #[test_case(
        [
        ("Both Eyes Distance CC", "20/20"),
        ("Both Eyes Distance CC +/-", "-2"),
        ],
        Ok([
        (("Both Eyes Distance CC".to_string()), Some(VisitNote {
            text: "20/20".to_string(),
            text_plus: "-2".to_string(),
            data_quality: DataQuality::Exact,
            laterality: Laterality::OU,
            distance_of_measurement: DistanceOfMeasurement::Distance,
            correction: Correction::CC,
            pinhole: PinHole::Unknown,
            va_format: Ok(VAFormat::Snellen),
            extracted_value: format ! ("20/20"),
            plus_letters: vec ! [- 2],
            snellen_equivalent: OptionResult::Some((20, 20).into()),
            log_mar_base: OptionResult::Some(0.0),
            log_mar_base_plus_letters: OptionResult::Some(0.0323),
        })),
        ])
    )]
    // #[test_case(
    //     [
    //     ("Visual Acuity Right Eye Distance CC", "20/100-1+2 ETDRS  (sc eccentric fixation)"),
    //     ],
    //     Ok([
    //     ("Visual Acuity Right Eye Distance CC".to_string(), Some(VisitNote {
    //         text: "20/100-1+2 ETDRS  (sc eccentric fixation)".to_string(),
    //         text_plus: "".to_string(),
    //         data_quality: DataQuality::ConvertibleFuzzy,
    //         laterality: Laterality::OD,
    //         distance_of_measurement: DistanceOfMeasurement::Distance,
    //         correction: Correction::Error(MultipleValues(format ! ("[CC, SC]"))),
    //         pinhole: PinHole::Unknown,
    //         va_format: VAFormat::Snellen,
    //         extracted_value: format ! ("20/100"),
    //         plus_letters: vec ! [- 1, 2],
    //         snellen_equivalent: OptionResult::Some((20, 100).into()),
    //         log_mar_base: OptionResult::Some(0.6990),
    //         log_mar_base_plus_letters: OptionResult::Some(0.6828),
    //     }))
    //     ])
    //     ; "Handling ambiguous VAFormats"
    // )]
    fn test_visit<'a, X: Into<VisitInput>, Y: Into<BTreeMap<String, Option<VisitNote>>>>(
        visit_notes: X,
        expected: R<Y>,
    ) {
        let expected = expected.map(|v| Visit(v.into()));
        let parser = Parser::new();
        let visit_notes = visit_notes.into();
        let actual = parser.parse_visit(visit_notes.clone());
        assert_almost_eq!(actual, expected, 4, "{visit_notes:?}");
    }

    #[test_case([("Visual Acuity", "20/20")], Ok(Correction::Unknown))]
    #[test_case([("Left Eye CC", "20/20")], Ok(CC))]
    #[test_case([("Left Eye SC", "20/20")], Ok(SC))]
    #[test_case([("Manifest Left Eye", "20/20")], Ok(Manifest))]
    // #[test_case([("Right Eye SC", "20/20 J5 CC")], Ok(Correction::Error(MultipleValues(s!("[SC, CC]")))))]
    fn test_visit_correction<'a, X>(visit_notes: X, expected: R<Correction>)
    where
        X: Into<VisitInput>,
    {
        test_visit_values(visit_notes, expected, |v: VisitNote| v.correction);
    }

    #[test_case([("Visual Acuity", "20/20")], Ok(DistanceOfMeasurement::Unknown))]
    #[test_case([("Both Eyes CC", "20/20")], Ok(DistanceOfMeasurement::Unknown))]
    #[test_case([("Both Eyes Distance CC", "20/20")], Ok(Distance))]
    #[test_case([("Both Eyes Near CC", "20/20")], Ok(Near))]
    #[test_case([("Manifest Both Eyes", "20/20")], Ok(Distance))]
    #[test_case([("Manifest Both Eyes Near", "20/20")], Ok(Near))]
    fn test_visit_distance_of_measurement<'a, X>(visit_notes: X, expected: R<DistanceOfMeasurement>)
    where
        X: Into<VisitInput>,
    {
        test_visit_values(visit_notes, expected, |v: VisitNote| {
            v.distance_of_measurement
        });
    }

    #[test_case([("Left Eye Distance SC", "20/30")], Ok(OS))]
    #[test_case([("Right Eye Distance SC", "20/30")], Ok(OD))]
    #[test_case([("Both Eyes Distance SC", "20/30")], Ok(OU))]
    // #[test_case(
    //     [("Left Eye Distance SC", "20/30 OU")],
    //     Ok(Laterality::Error(MultipleValues(s ! ("[OS, OU]"))))
    // )]
    fn test_visit_laterality<'a, X>(visit_notes: X, expected: R<Laterality>)
    where
        X: Into<VisitInput>,
    {
        test_visit_values(visit_notes, expected, |v: VisitNote| v.laterality);
    }

    #[test_case([("Visual Acuity", "20/30 OS")], Ok(ConvertibleFuzzy))]
    #[test_case([("Visual Acuity", "CSM")], Ok(Exact))]
    #[test_case([("Visual Acuity", "CSM pref")], Ok(Exact))]
    fn test_visit_data_quality<'a, X>(visit_notes: X, expected: R<DataQuality>)
    where
        X: Into<VisitInput>,
    {
        test_visit_values(visit_notes, expected, |v: VisitNote| v.data_quality);
    }

    fn test_visit_values<V, T, F>(visit_notes: V, expected: R<T>, f: F)
    where
        V: Into<VisitInput>,
        T: PartialEq + Debug,
        F: Fn(VisitNote) -> T,
    {
        let expected = expected.map(|v| vec![v]);
        let visit_notes = visit_notes.into();

        let actual = Parser::new().parse_visit(visit_notes.clone()).map(|r| {
            r.0.into_iter()
                .map(|(_, v)| f(v.expect("TEST")))
                .collect_vec()
        });

        assert_eq!(actual, expected, "{visit_notes:?}");
    }
}
