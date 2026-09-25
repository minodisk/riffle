//! Display formatting of the shooting settings, shared by the metadata pane
//! and the index.

use riffle_core::arw::{Rational, Shot};

/// A numeric setting: `value` to sort by, `label` to show.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct Labeled {
    pub value: f64,
    pub label: String,
}

/// The shooting settings of one file, formatted for display. Fields the file
/// does not carry are `None`.
#[derive(Debug, Clone, Default, PartialEq, serde::Serialize)]
pub struct Exif {
    pub camera: Option<String>,
    pub lens: Option<String>,
    pub aperture: Option<Labeled>,
    pub shutter: Option<Labeled>,
    pub iso: Option<Labeled>,
    pub focal_length: Option<Labeled>,
    pub focus_mode: Option<String>,
    pub af_tracking: Option<String>,
    pub af_area: Option<String>,
}

/// Format a rational as a decimal with at most `places` digits, with trailing
/// zeros dropped (2.80 -> "2.8", 50.0 -> "50").
pub fn decimal(r: Rational, places: usize) -> Option<String> {
    let v = r.value()?;
    let text = format!("{v:.places$}");
    let text = if text.contains('.') {
        text.trim_end_matches('0').trim_end_matches('.')
    } else {
        text.as_str()
    };
    Some(text.to_string())
}

/// Shutter speed the way a camera shows it: `1/250` below a second, `1.3"`
/// at or above one.
fn shutter(r: Rational) -> Option<String> {
    let v = r.value()?;
    if v <= 0.0 {
        return None;
    }
    if v >= 1.0 {
        return decimal(r, 1).map(|t| format!("{t}\""));
    }
    Some(format!("1/{}", (1.0 / v).round()))
}

/// Sony `FocusMode` (0x201b), labeled as ExifTool's Sony.pm `%Sony::Main`.
fn focus_mode(v: u8) -> Option<&'static str> {
    match v {
        0 => Some("Manual"),
        2 => Some("AF-S"),
        3 => Some("AF-C"),
        4 => Some("AF-A"),
        6 => Some("DMF"),
        7 => Some("AF-D"),
        _ => None,
    }
}

/// Sony `AFTracking` (0x2021), labeled as ExifTool's Sony.pm `%Sony::Main`.
fn af_tracking(v: u8) -> Option<&'static str> {
    match v {
        0 => Some("Off"),
        1 => Some("Face tracking"),
        2 => Some("Lock-On AF"),
        _ => None,
    }
}

/// Sony `AFAreaModeSetting` (0x201c), labeled with ExifTool's Sony.pm
/// NEX/ILCE/ZV table. The SLT/HV and ILCA tables are intentionally not
/// mapped, and the `ILME-` and RX/HX `DSC-` bodies ExifTool also reads with
/// this table are deliberately left out.
fn af_area(model: Option<&str>, v: u8) -> Option<&'static str> {
    let model = model?;
    if !["ILCE-", "NEX-", "ZV-"]
        .iter()
        .any(|prefix| model.starts_with(prefix))
    {
        return None;
    }
    match v {
        0 => Some("Wide"),
        1 => Some("Center"),
        3 => Some("Flexible Spot"),
        4 => Some("Flexible Spot (LA-EA4)"),
        9 => Some("Center (LA-EA4)"),
        11 => Some("Zone"),
        12 => Some("Expanded Flexible Spot"),
        13 => Some("Custom AF Area"),
        _ => None,
    }
}

pub fn exif(shot: &Shot) -> Exif {
    // The model usually already starts with the make ("SONY" / "ILCE-7M5"),
    // so the two are joined rather than one being dropped.
    let camera = match (shot.make.as_deref(), shot.model.as_deref()) {
        (Some(make), Some(model)) => Some(format!("{make} {model}")),
        (make, model) => make.or(model).map(str::to_string),
    };
    let aperture = match (shot.f_number, shot.estimated_f_number) {
        (Some(r), _) => r.value().zip(decimal(r, 1)).map(|(value, t)| Labeled {
            value,
            label: format!("f/{t}"),
        }),
        (None, Some(f)) => decimal(
            Rational {
                num: (f * 10.0).round() as i64,
                den: 10,
            },
            1,
        )
        .map(|t| Labeled {
            value: f,
            label: format!("f/{t} (est.)"),
        }),
        (None, None) => None,
    };
    Exif {
        camera,
        lens: shot.lens_model.clone(),
        aperture,
        shutter: shot.exposure_time.and_then(|r| {
            Some(Labeled {
                label: shutter(r)?,
                value: r.value()?,
            })
        }),
        iso: shot.iso.map(|v| Labeled {
            value: f64::from(v),
            label: v.to_string(),
        }),
        focal_length: shot.focal_length.and_then(|r| {
            Some(Labeled {
                label: format!("{} mm", decimal(r, 1)?),
                value: r.value()?,
            })
        }),
        focus_mode: shot.focus_mode.and_then(focus_mode).map(str::to_string),
        af_tracking: shot.af_tracking.and_then(af_tracking).map(str::to_string),
        af_area: shot
            .af_area_mode
            .and_then(|v| af_area(shot.model.as_deref(), v))
            .map(str::to_string),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn r(num: i64, den: i64) -> Rational {
        Rational { num, den }
    }

    fn camera(make: Option<&str>, model: Option<&str>) -> Option<String> {
        exif(&Shot {
            make: make.map(str::to_string),
            model: model.map(str::to_string),
            ..Shot::default()
        })
        .camera
    }

    #[test]
    fn shutter_reads_as_a_fraction_below_a_second_and_seconds_above() {
        assert_eq!(shutter(r(1, 250)).as_deref(), Some("1/250"));
        assert_eq!(shutter(r(13, 10)).as_deref(), Some("1.3\""));
        assert_eq!(shutter(r(4, 1)).as_deref(), Some("4\""));
        assert_eq!(shutter(r(0, 1)), None);
        assert_eq!(shutter(r(1, 0)), None);
    }

    #[test]
    fn decimals_drop_their_trailing_zeros() {
        assert_eq!(decimal(r(28, 10), 1).as_deref(), Some("2.8"));
        assert_eq!(decimal(r(500, 10), 1).as_deref(), Some("50"));
        assert_eq!(decimal(r(1, 0), 1), None);
        assert_eq!(decimal(r(100, 1), 0).as_deref(), Some("100"));
    }

    #[test]
    fn the_numeric_settings_carry_their_value_and_label() {
        let e = exif(&Shot {
            exposure_time: Some(r(1, 250)),
            f_number: Some(r(28, 10)),
            iso: Some(800),
            focal_length: Some(r(500, 10)),
            lens_model: Some("FE 50mm F1.4 GM".to_string()),
            ..Shot::default()
        });
        let label = |l: &Option<Labeled>| l.as_ref().map(|l| (l.value, l.label.clone()));
        assert_eq!(label(&e.shutter), Some((0.004, "1/250".to_string())));
        assert_eq!(label(&e.aperture), Some((2.8, "f/2.8".to_string())));
        assert_eq!(label(&e.iso), Some((800.0, "800".to_string())));
        assert_eq!(label(&e.focal_length), Some((50.0, "50 mm".to_string())));
        assert_eq!(e.lens.as_deref(), Some("FE 50mm F1.4 GM"));
    }

    #[test]
    fn an_estimated_aperture_is_marked() {
        let e = exif(&Shot {
            estimated_f_number: Some(2.0),
            ..Shot::default()
        });
        let a = e.aperture.unwrap();
        assert_eq!(a.value, 2.0);
        assert_eq!(a.label, "f/2 (est.)");
    }

    #[test]
    fn the_camera_joins_make_and_model() {
        assert_eq!(
            camera(Some("SONY"), Some("ILCE-7M5")).as_deref(),
            Some("SONY ILCE-7M5")
        );
        assert_eq!(camera(Some("SONY"), None).as_deref(), Some("SONY"));
        assert_eq!(camera(None, Some("ILCE-7M5")).as_deref(), Some("ILCE-7M5"));
        assert_eq!(camera(None, None), None);
    }

    #[test]
    fn the_sony_focus_mode_maps_to_its_label() {
        for (v, label) in [
            (0, "Manual"),
            (2, "AF-S"),
            (3, "AF-C"),
            (4, "AF-A"),
            (6, "DMF"),
            (7, "AF-D"),
        ] {
            assert_eq!(focus_mode(v), Some(label));
        }
        for v in [1, 5, 255] {
            assert_eq!(focus_mode(v), None);
        }
    }

    #[test]
    fn the_sony_af_tracking_maps_to_its_label() {
        assert_eq!(af_tracking(0), Some("Off"));
        assert_eq!(af_tracking(1), Some("Face tracking"));
        assert_eq!(af_tracking(2), Some("Lock-On AF"));
        assert_eq!(af_tracking(3), None);
    }

    #[test]
    fn the_sony_af_area_maps_to_its_label_on_ilce_nex_and_zv_bodies() {
        for model in ["ILCE-7M5", "NEX-7", "ZV-E10"] {
            for (v, label) in [
                (0, "Wide"),
                (1, "Center"),
                (3, "Flexible Spot"),
                (4, "Flexible Spot (LA-EA4)"),
                (9, "Center (LA-EA4)"),
                (11, "Zone"),
                (12, "Expanded Flexible Spot"),
                (13, "Custom AF Area"),
            ] {
                assert_eq!(af_area(Some(model), v), Some(label));
            }
            for v in [2, 8, 255] {
                assert_eq!(af_area(Some(model), v), None);
            }
        }
    }

    #[test]
    fn the_sony_af_area_is_left_out_on_other_bodies() {
        for model in [
            Some("ILME-FX3"),
            Some("ILCA-99M2"),
            Some("SLT-A99V"),
            Some("DSC-RX100M7"),
            Some("LEICA M11-P"),
            None,
        ] {
            assert_eq!(af_area(model, 0), None);
        }
    }

    #[test]
    fn the_af_fields_are_formatted_from_the_shot() {
        let e = exif(&Shot {
            model: Some("ILCE-7M5".to_string()),
            focus_mode: Some(3),
            af_tracking: Some(1),
            af_area_mode: Some(13),
            ..Shot::default()
        });
        assert_eq!(e.focus_mode.as_deref(), Some("AF-C"));
        assert_eq!(e.af_tracking.as_deref(), Some("Face tracking"));
        assert_eq!(e.af_area.as_deref(), Some("Custom AF Area"));
    }

    #[test]
    fn a_shot_without_settings_formats_to_nothing() {
        assert_eq!(exif(&Shot::default()), Exif::default());
    }
}
