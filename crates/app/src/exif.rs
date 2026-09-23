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
    fn a_shot_without_settings_formats_to_nothing() {
        assert_eq!(exif(&Shot::default()), Exif::default());
    }
}
