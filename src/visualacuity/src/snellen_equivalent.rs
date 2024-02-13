use crate::*;
use crate::ParsedItem::*;
use crate::LowVisionMethod::*;
use crate::JaegerRow::*;
use crate::VisualAcuityError::*;

pub(crate) trait SnellenEquivalent {
    fn snellen_equivalent(&self) -> VisualAcuityResult<(u16, u16)>;
}

impl SnellenEquivalent for ParsedItem {
    fn snellen_equivalent(&self) -> VisualAcuityResult<(u16, u16)> {
        match self {
            Snellen(row) => Ok((20, *row as u16)),
            Jaeger(row) => match row {
                // https://www.healio.com/~/media/Files/Journals/General%20PDFs/JRS/JRSVACHART.ashx
                J1 => Ok((20, 20)),
                J2 => Ok((20, 25)),
                J3 => Ok((20, 30)),
                J4 => Ok((20, 32)),
                J5 => Ok((20, 40)),
                J6 => Ok((20, 50)),
                J7 => Ok((20, 60)),
                J8 => Ok((20, 63)),
                J9 => Ok((20, 80)),
                J10 => Ok((20, 100)),
                J11 => Ok((20, 114)),
                J12 => Ok((20, 125)),
                J13 => Ok((20, 160)),
                J14 => Ok((20, 200)),
                // TODO: figure out other Snellen values!
                _ => Err(NoSnellenEquivalent),
            },
            ETDRS { letters } => match letters {
                // https://www.researchgate.net/figure/Conversions-Between-Letter-LogMAR-and-Snellen-Visual-Acuity-Scores_tbl1_258819613
                0|1|2|3|4|5 => Ok((20, 800)),
                6|7|8|9|10 => Ok((20, 640)),
                11|12|13|14|15 => Ok((20, 500)),
                16|17|18|19|20 => Ok((20, 400)),
                21|22|23|24|25 => Ok((20, 320)),
                26|27|28|29|30 => Ok((20, 250)),
                31|32|33|34|35 => Ok((20, 200)),
                36|37|38|39|40 => Ok((20, 160)),
                41|42|43|44|45 => Ok((20, 125)),
                46|47|48|49|50 => Ok((20, 100)),
                51|52|53|54|55 => Ok((20, 80)),
                56|57|58|59|60 => Ok((20, 63)),
                61|62|63|64|65 => Ok((20, 50)),
                66|67|68|69|70 => Ok((20, 40)),
                71|72|73|74|75 => Ok((20, 32)),
                76|77|78|79|80 => Ok((20, 25)),
                81|82|83|84|85 => Ok((20, 20)),
                86|87|88|89|90 => Ok((20, 15)),
                91|92|93|94|95 => Ok((20, 12)),
                _ => Err(NoSnellenEquivalent)
            },
            Teller { row, .. } => Ok((20, *row)),
            LowVision { method, distance } => match distance.to_feet() {
                Ok(distance) => {
                    use crate::DistanceUnits::*;
                    fn convert_using_reference_acuity(feet: f64, ref_size: f64, ref_feet: f64) -> (u16, u16) {
                        (20, (ref_size * ref_feet / feet) as u16)
                    }

                    // let citation = "Bach et al. (2007)";
                    let citation = "Schulze-Bonzel et al. 2006";
                    let (ref_size, ref_distance) = match citation {
                        "Holladay (1997)" => match method {
                            CountingFingers => Ok((200, Feet(20.0))),
                            HandMovement => Ok((2000, Feet(20.0))),
                            LightPerception => Err(NoSnellenEquivalent),
                            NoLightPerception => Err(NoSnellenEquivalent),
                        },
                        "Schulze-Bonzel et al. 2006" => match method {
                            CountingFingers => Ok((1500, Centimeters(30.0))),
                            HandMovement => Ok((4000, Centimeters(30.0))),
                            LightPerception => Err(NoSnellenEquivalent),
                            NoLightPerception => Err(NoSnellenEquivalent),
                        },
                        "Bach et al. (2007)" => match method {
                            CountingFingers => Ok((1500, Centimeters(30.0))),
                            HandMovement => Ok((4000, Centimeters(30.0))),
                            LightPerception => Err(NoSnellenEquivalent),
                            NoLightPerception => Err(NoSnellenEquivalent),
                        },
                        _ => Err(NotImplemented)
                    }?;
                    let (ref_size, ref_feet) = (ref_size as f64, ref_distance.to_feet()?);
                    Ok(convert_using_reference_acuity(distance, ref_size, ref_feet))
                }
                Err(e) => Err(e)
            },
            _ => Err(NoSnellenEquivalent),
        }
    }
}
