//! XMP sidecar reading and writing for ratings and colour labels.
//!
//! A rating is a single signed integer in `-1..=5`: `0` is unrated, `1`-`5`
//! are stars and `-1` is a reject, which is the convention Adobe Bridge
//! writes and darktable reads. A label is the raw string of `xmp:Label`,
//! kept as-is so any vocabulary round-trips. Nothing but `xmp:Rating` and
//! `xmp:Label` is ever written.
//!
//! An existing sidecar is patched by splicing the bytes of the `Rating` or
//! `Label` value (or removing the `Label` property), so a Lightroom sidecar
//! keeps its `crs:` develop settings byte-for-byte; a sidecar is never
//! regenerated from a parse.

use std::path::{Path, PathBuf};

use quick_xml::events::Event;
use quick_xml::name::ResolveResult;
use quick_xml::NsReader;

const XMP_NS: &str = "http://ns.adobe.com/xap/1.0/";
const RDF_NS: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#";

/// The sidecar path of an ARW: the extension replaced with `xmp`.
///
/// Pure by design: when the directory already holds a sidecar differing only
/// in case (`FOO.XMP`), the caller is the one that lists the directory and
/// prefers the existing name.
pub fn sidecar_path(arw: &Path) -> PathBuf {
    let mut path = arw.to_path_buf();
    path.set_extension("xmp");
    path
}

/// The `xmp:Rating` of the first `rdf:Description` that has one, clamped to
/// `-1..=5`.
///
/// `None` when the property is absent or its value is outside the range;
/// `Err` when the bytes are not parseable XMP.
pub fn read_rating(bytes: &[u8]) -> Result<Option<i8>, String> {
    let text = std::str::from_utf8(bytes).map_err(|e| format!("not UTF-8: {e}"))?;
    match locate(text, "Rating")? {
        Location::Value { start, end, .. } => Ok(text[start..end]
            .trim()
            .parse::<i8>()
            .ok()
            .filter(|n| (-1..=5).contains(n))),
        Location::Insert { .. } => Ok(None),
    }
}

/// The sidecar bytes carrying `rating`.
///
/// With `existing` `None` this is a fresh template; otherwise the existing
/// bytes with only the `Rating` value replaced in the first `rdf:Description`
/// that has one, or one attribute inserted into the first `rdf:Description`
/// when the property is absent from all of them.
///
/// `rating` `None` means unrated and writes `0`. On a file that has no
/// sidecar yet the caller must not write anything at all rather than call
/// this with `None`, so that clearing a rating does not litter a folder with
/// empty sidecars.
pub fn write_rating(existing: Option<&[u8]>, rating: Option<i8>) -> Result<Vec<u8>, String> {
    let value = rating.unwrap_or(0);
    if !(-1..=5).contains(&value) {
        return Err(format!("rating {value} is outside -1..=5"));
    }
    let Some(existing) = existing else {
        return Ok(template("Rating", &value.to_string()).into_bytes());
    };
    set(existing, "Rating", &value.to_string())
}

/// The `xmp:Label` of the first `rdf:Description` that has one, as the raw
/// string the sidecar holds.
///
/// `None` when the property is absent or empty; `Err` when the bytes are not
/// parseable XMP.
pub fn read_label(bytes: &[u8]) -> Result<Option<String>, String> {
    let text = std::str::from_utf8(bytes).map_err(|e| format!("not UTF-8: {e}"))?;
    match locate(text, "Label")? {
        Location::Value { start, end, .. } if !text[start..end].trim().is_empty() => {
            Ok(Some(text[start..end].to_string()))
        }
        _ => Ok(None),
    }
}

/// The sidecar bytes carrying `label`.
///
/// `Some` splices the value in place or inserts one attribute, as
/// [`write_rating`] does; `None` removes the property and leaves a sidecar
/// without one byte-identical. With `existing` `None`, `Some` is a fresh
/// template and `None` is an error: no sidecar is minted for "no label".
pub fn write_label(existing: Option<&[u8]>, label: Option<&str>) -> Result<Vec<u8>, String> {
    let Some(existing) = existing else {
        return match label {
            Some(label) => Ok(template("Label", label).into_bytes()),
            None => Err("no sidecar to clear a label from".to_string()),
        };
    };
    let Some(label) = label else {
        let text = std::str::from_utf8(existing).map_err(|e| format!("not UTF-8: {e}"))?;
        return Ok(match locate(text, "Label")? {
            Location::Value { remove, .. } => {
                format!("{}{}", &text[..remove.0], &text[remove.1..]).into_bytes()
            }
            Location::Insert { .. } => existing.to_vec(),
        });
    };
    set(existing, "Label", label)
}

fn set(existing: &[u8], name: &str, value: &str) -> Result<Vec<u8>, String> {
    let text = std::str::from_utf8(existing).map_err(|e| format!("not UTF-8: {e}"))?;
    let mut out = String::with_capacity(text.len() + 32);
    match locate(text, name)? {
        Location::Value { start, end, .. } => {
            out.push_str(&text[..start]);
            out.push_str(value);
            out.push_str(&text[end..]);
        }
        Location::Insert {
            at,
            prefix,
            declare,
        } => {
            out.push_str(&text[..at]);
            if declare {
                out.push_str(&format!(" xmlns:{prefix}=\"{XMP_NS}\""));
            }
            out.push_str(&format!(" {prefix}:{name}=\"{value}\""));
            out.push_str(&text[at..]);
        }
    }
    Ok(out.into_bytes())
}

fn template(name: &str, value: &str) -> String {
    format!(
        "<?xpacket begin=\"\" id=\"W5M0MpCehiHzreSzNTczkc9d\"?>\n\
         <x:xmpmeta xmlns:x=\"adobe:ns:meta/\" x:xmptk=\"riffle\">\n\
         \x20<rdf:RDF xmlns:rdf=\"{RDF_NS}\">\n\
         \x20 <rdf:Description rdf:about=\"\" xmlns:xmp=\"{XMP_NS}\" xmp:{name}=\"{value}\"/>\n\
         \x20</rdf:RDF>\n\
         </x:xmpmeta>\n\
         <?xpacket end=\"w\"?>\n"
    )
}

enum Location {
    /// The byte range of the property's value, attribute or element text,
    /// and the range removing the whole property: the attribute with the
    /// whitespace before it, or the element (with its line when alone on it).
    Value {
        start: usize,
        end: usize,
        remove: (usize, usize),
    },
    /// Where to insert an attribute in the first `rdf:Description` start tag.
    Insert {
        at: usize,
        prefix: String,
        declare: bool,
    },
}

fn locate(text: &str, name: &str) -> Result<Location, String> {
    let mut reader = NsReader::from_str(text);
    let mut pos = 0usize;
    let mut insert: Option<Location> = None;
    let mut first_description_seen = false;
    let mut element: Option<(usize, usize)> = None;
    loop {
        let event = reader
            .read_event()
            .map_err(|e| format!("XMP parse error: {e}"))?;
        let end = reader.buffer_position() as usize;
        let span = &text[pos..end];
        match &event {
            Event::Eof => break,
            Event::Start(e) | Event::Empty(e) => {
                let (ns, local) = reader.resolver().resolve_element(e.name());
                if bound_to(&ns, RDF_NS) && local.as_ref() == "Description" {
                    for attr in e.attributes() {
                        let attr = attr.map_err(|e| format!("XMP parse error: {e}"))?;
                        let (ans, alocal) = reader.resolver().resolve_attribute(attr.key);
                        if bound_to(&ans, XMP_NS) && alocal.as_ref() == name {
                            if let Some((k, s, e)) = attr_value_range(span, attr.key.as_ref()) {
                                let lead = span[..k].trim_end().len();
                                return Ok(Location::Value {
                                    start: pos + s,
                                    end: pos + e,
                                    remove: (pos + lead, pos + e + 1),
                                });
                            }
                        }
                    }
                    if !first_description_seen {
                        first_description_seen = true;
                        let (prefix, declare) = xmp_prefix(&reader);
                        insert = Some(Location::Insert {
                            at: pos + insert_offset(span),
                            prefix,
                            declare,
                        });
                    }
                } else if bound_to(&ns, XMP_NS) && local.as_ref() == name {
                    element = matches!(event, Event::Start(_)).then_some((pos, end));
                }
            }
            Event::End(e) => {
                let (ns, local) = reader.resolver().resolve_element(e.name());
                if bound_to(&ns, XMP_NS) && local.as_ref() == name {
                    if let Some((open, open_tag_end)) = element {
                        // The value range spans everything between the open
                        // and close tags, `open_tag_end..pos` (`pos` is the
                        // start of this End event), so entity references such
                        // as `&amp;` inside the text stay part of the value
                        // instead of being cut at the first `Text` event.
                        return Ok(Location::Value {
                            start: open_tag_end,
                            end: pos,
                            remove: line_or_element(text, open, end),
                        });
                    }
                }
                element = None;
            }
            _ => {}
        }
        pos = end;
    }
    insert.ok_or_else(|| "no rdf:Description element".to_string())
}

fn bound_to(ns: &ResolveResult<'_>, expected: &str) -> bool {
    matches!(ns, ResolveResult::Bound(n) if n.as_ref() == expected)
}

/// The whole line when the element `open..close` sits alone on it, otherwise
/// just the element.
fn line_or_element(text: &str, open: usize, close: usize) -> (usize, usize) {
    let line_start = text[..open].rfind('\n').map_or(0, |i| i + 1);
    let Some(nl) = text[close..].find('\n') else {
        return (open, close);
    };
    if text[line_start..open].trim().is_empty() && text[close..close + nl].trim().is_empty() {
        (line_start, close + nl + 1)
    } else {
        (open, close)
    }
}

/// The prefix to write a property with, and whether it has to be declared.
fn xmp_prefix(reader: &NsReader<&[u8]>) -> (String, bool) {
    for candidate in ["xmp", "xap"] {
        let name = format!("{candidate}:Rating");
        let (ns, _) = reader
            .resolver()
            .resolve_attribute(quick_xml::name::QName(&name));
        if bound_to(&ns, XMP_NS) {
            return (candidate.to_string(), false);
        }
    }
    ("xmp".to_string(), true)
}

/// The offset of `key` and the byte range of its value inside a start tag,
/// quotes excluded.
///
/// quick-xml reports positions per event, not per attribute, so the value is
/// located by scanning the start tag's own range, which is exact.
fn attr_value_range(tag: &str, key: &str) -> Option<(usize, usize, usize)> {
    let mut from = 0usize;
    while let Some(rel) = tag[from..].find(key) {
        let start = from + rel;
        from = start + key.len();
        if !tag[..start]
            .chars()
            .next_back()
            .is_some_and(char::is_whitespace)
        {
            continue;
        }
        let after_key = tag[from..].trim_start();
        let Some(after_eq) = after_key.strip_prefix('=') else {
            continue;
        };
        let after_eq = after_eq.trim_start();
        let Some(quote) = after_eq.chars().next() else {
            continue;
        };
        if quote != '"' && quote != '\'' {
            continue;
        }
        let value_start = tag.len() - after_eq.len() + 1;
        let value_end = value_start + tag[value_start..].find(quote)?;
        return Some((start, value_start, value_end));
    }
    None
}

/// Where an attribute can be appended in a start tag: before `>` or `/>`.
fn insert_offset(tag: &str) -> usize {
    let close = tag.rfind('>').unwrap_or(tag.len());
    let before = tag[..close].trim_end();
    if before.ends_with('/') {
        before.len() - 1
    } else {
        close
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const BRIDGE: &str = concat!(
        "<?xpacket begin=\"\" id=\"W5M0MpCehiHzreSzNTczkc9d\"?>\n",
        "<x:xmpmeta xmlns:x=\"adobe:ns:meta/\" x:xmptk=\"Adobe XMP Core\">\n",
        " <rdf:RDF xmlns:rdf=\"http://www.w3.org/1999/02/22-rdf-syntax-ns#\">\n",
        "  <rdf:Description rdf:about=\"\"\n",
        "    xmlns:xmp=\"http://ns.adobe.com/xap/1.0/\"\n",
        "   xmp:Rating=\"3\"\n",
        "   xmp:CreatorTool=\"Bridge\"/>\n",
        " </rdf:RDF>\n",
        "</x:xmpmeta>\n",
        "<?xpacket end=\"w\"?>\n",
    );

    const LIGHTROOM: &str = concat!(
        "<?xpacket begin=\"\" id=\"W5M0MpCehiHzreSzNTczkc9d\"?>\n",
        "<x:xmpmeta xmlns:x=\"adobe:ns:meta/\" x:xmptk=\"Adobe XMP Core 9.0\">\n",
        " <rdf:RDF xmlns:rdf=\"http://www.w3.org/1999/02/22-rdf-syntax-ns#\">\n",
        "  <rdf:Description rdf:about=\"\"\n",
        "    xmlns:xmp=\"http://ns.adobe.com/xap/1.0/\"\n",
        "    xmlns:dc=\"http://purl.org/dc/elements/1.1/\"\n",
        "    xmlns:crs=\"http://ns.adobe.com/camera-raw-settings/1.0/\"\n",
        "    crs:Version=\"15.0\"\n",
        "    crs:ProcessVersion=\"11.0\"\n",
        "    crs:WhiteBalance=\"As Shot\"\n",
        "    crs:Temperature=\"5200\"\n",
        "    crs:Tint=\"+8\"\n",
        "    crs:Exposure2012=\"+0.35\"\n",
        "    crs:Contrast2012=\"+12\"\n",
        "    crs:Highlights2012=\"-40\"\n",
        "    crs:Shadows2012=\"+25\"\n",
        "    crs:ToneCurveName2012=\"Medium Contrast\">\n",
        "   <xmp:Rating>2</xmp:Rating>\n",
        "   <xmp:CreateDate>2026-09-18T10:00:00</xmp:CreateDate>\n",
        "   <dc:subject>\n",
        "    <rdf:Bag>\n",
        "     <rdf:li>game</rdf:li>\n",
        "     <rdf:li>keeper</rdf:li>\n",
        "    </rdf:Bag>\n",
        "   </dc:subject>\n",
        "  </rdf:Description>\n",
        " </rdf:RDF>\n",
        "</x:xmpmeta>\n",
        "<?xpacket end=\"w\"?>\n",
    );

    const NO_RATING: &str = concat!(
        "<x:xmpmeta xmlns:x=\"adobe:ns:meta/\">\n",
        " <rdf:RDF xmlns:rdf=\"http://www.w3.org/1999/02/22-rdf-syntax-ns#\">\n",
        "  <rdf:Description rdf:about=\"\" xmlns:xmp=\"http://ns.adobe.com/xap/1.0/\"\n",
        "   xmp:CreatorTool=\"darktable\"/>\n",
        " </rdf:RDF>\n",
        "</x:xmpmeta>\n",
    );

    fn patched(source: &str, rating: Option<i8>) -> String {
        String::from_utf8(write_rating(Some(source.as_bytes()), rating).unwrap()).unwrap()
    }

    #[test]
    fn sidecar_path_replaces_the_extension() {
        assert_eq!(
            sidecar_path(Path::new("/photos/FOO.ARW")),
            PathBuf::from("/photos/FOO.xmp")
        );
    }

    #[test]
    fn reads_an_attribute_sidecar() {
        assert_eq!(read_rating(BRIDGE.as_bytes()).unwrap(), Some(3));
    }

    #[test]
    fn reads_an_element_sidecar() {
        assert_eq!(read_rating(LIGHTROOM.as_bytes()).unwrap(), Some(2));
    }

    #[test]
    fn patches_only_the_attribute_value() {
        let out = patched(BRIDGE, Some(5));
        assert_eq!(out, BRIDGE.replace("xmp:Rating=\"3\"", "xmp:Rating=\"5\""));
        assert_eq!(read_rating(out.as_bytes()).unwrap(), Some(5));
    }

    #[test]
    fn patches_only_the_element_text() {
        let out = patched(LIGHTROOM, Some(-1));
        assert_eq!(
            out,
            LIGHTROOM.replace("<xmp:Rating>2</xmp:Rating>", "<xmp:Rating>-1</xmp:Rating>")
        );
        assert_eq!(read_rating(out.as_bytes()).unwrap(), Some(-1));
    }

    #[test]
    fn clearing_writes_zero_into_an_existing_sidecar() {
        assert_eq!(
            read_rating(patched(LIGHTROOM, None).as_bytes()).unwrap(),
            Some(0)
        );
    }

    #[test]
    fn inserts_exactly_one_attribute_when_absent() {
        let out = patched(NO_RATING, Some(4));
        assert_eq!(
            out,
            NO_RATING.replace(
                "xmp:CreatorTool=\"darktable\"/>",
                "xmp:CreatorTool=\"darktable\" xmp:Rating=\"4\"/>"
            )
        );
        assert_eq!(read_rating(out.as_bytes()).unwrap(), Some(4));
    }

    #[test]
    fn declares_the_namespace_when_the_description_has_none() {
        let source = concat!(
            "<rdf:RDF xmlns:rdf=\"http://www.w3.org/1999/02/22-rdf-syntax-ns#\">\n",
            " <rdf:Description rdf:about=\"\"></rdf:Description>\n",
            "</rdf:RDF>\n",
        );
        let out = patched(source, Some(1));
        assert!(out.contains(
            "<rdf:Description rdf:about=\"\" xmlns:xmp=\"http://ns.adobe.com/xap/1.0/\" xmp:Rating=\"1\">"
        ));
        assert_eq!(read_rating(out.as_bytes()).unwrap(), Some(1));
    }

    #[test]
    fn handles_the_xap_prefix() {
        let source = concat!(
            "<rdf:RDF xmlns:rdf=\"http://www.w3.org/1999/02/22-rdf-syntax-ns#\"\n",
            "  xmlns:xap=\"http://ns.adobe.com/xap/1.0/\">\n",
            " <rdf:Description rdf:about=\"\" xap:Rating=\"1\"/>\n",
            "</rdf:RDF>\n",
        );
        assert_eq!(read_rating(source.as_bytes()).unwrap(), Some(1));
        assert_eq!(
            patched(source, Some(-1)),
            source.replace("xap:Rating=\"1\"", "xap:Rating=\"-1\"")
        );
    }

    #[test]
    fn handles_a_default_namespace_description() {
        let source = concat!(
            "<RDF xmlns=\"http://www.w3.org/1999/02/22-rdf-syntax-ns#\"\n",
            "  xmlns:xmp=\"http://ns.adobe.com/xap/1.0/\">\n",
            " <Description about=\"\">\n",
            "  <xmp:Rating>4</xmp:Rating>\n",
            " </Description>\n",
            "</RDF>\n",
        );
        assert_eq!(read_rating(source.as_bytes()).unwrap(), Some(4));
        assert_eq!(
            patched(source, Some(0)),
            source.replace("<xmp:Rating>4<", "<xmp:Rating>0<")
        );
    }

    #[test]
    fn an_out_of_range_value_reads_as_absent() {
        let source = BRIDGE.replace("xmp:Rating=\"3\"", "xmp:Rating=\"9\"");
        assert_eq!(read_rating(source.as_bytes()).unwrap(), None);
    }

    #[test]
    fn a_sidecar_without_a_rating_reads_as_none() {
        assert_eq!(read_rating(NO_RATING.as_bytes()).unwrap(), None);
    }

    #[test]
    fn finds_a_rating_on_a_sibling_description() {
        let source = concat!(
            "<rdf:RDF xmlns:rdf=\"http://www.w3.org/1999/02/22-rdf-syntax-ns#\">\n",
            " <rdf:Description rdf:about=\"\"\n",
            "   xmlns:tiff=\"http://ns.adobe.com/tiff/1.0/\"\n",
            "   tiff:Make=\"SONY\"/>\n",
            " <rdf:Description rdf:about=\"\"\n",
            "   xmlns:xmp=\"http://ns.adobe.com/xap/1.0/\"\n",
            "   xmp:Rating=\"4\"/>\n",
            "</rdf:RDF>\n",
        );
        assert_eq!(read_rating(source.as_bytes()).unwrap(), Some(4));
        assert_eq!(
            patched(source, Some(2)),
            source.replace("xmp:Rating=\"4\"", "xmp:Rating=\"2\"")
        );
    }

    #[test]
    fn splices_an_empty_element_rating_in_place() {
        let source = concat!(
            "<rdf:RDF xmlns:rdf=\"http://www.w3.org/1999/02/22-rdf-syntax-ns#\">\n",
            " <rdf:Description rdf:about=\"\"\n",
            "   xmlns:xmp=\"http://ns.adobe.com/xap/1.0/\">\n",
            "  <xmp:Rating></xmp:Rating>\n",
            " </rdf:Description>\n",
            "</rdf:RDF>\n",
        );
        let out = patched(source, Some(2));
        assert_eq!(
            out,
            source.replace("<xmp:Rating></xmp:Rating>", "<xmp:Rating>2</xmp:Rating>")
        );
        assert_eq!(read_rating(out.as_bytes()).unwrap(), Some(2));
    }

    #[test]
    fn a_truncated_document_is_an_error() {
        let source = &BRIDGE[..BRIDGE.len() / 2];
        assert!(read_rating(source.as_bytes()).is_err());
        assert!(write_rating(Some(source.as_bytes()), Some(1)).is_err());
    }

    #[test]
    fn a_document_without_a_description_is_an_error() {
        assert!(read_rating(b"<html><body>hello</body></html>").is_err());
    }

    #[test]
    fn a_fresh_template_round_trips() {
        for n in -1..=5 {
            let bytes = write_rating(None, Some(n)).unwrap();
            assert_eq!(read_rating(&bytes).unwrap(), Some(n));
        }
    }

    #[test]
    fn the_fresh_template_is_the_documented_one() {
        assert_eq!(
            String::from_utf8(write_rating(None, Some(3)).unwrap()).unwrap(),
            concat!(
                "<?xpacket begin=\"\" id=\"W5M0MpCehiHzreSzNTczkc9d\"?>\n",
                "<x:xmpmeta xmlns:x=\"adobe:ns:meta/\" x:xmptk=\"riffle\">\n",
                " <rdf:RDF xmlns:rdf=\"http://www.w3.org/1999/02/22-rdf-syntax-ns#\">\n",
                "  <rdf:Description rdf:about=\"\" xmlns:xmp=\"http://ns.adobe.com/xap/1.0/\" xmp:Rating=\"3\"/>\n",
                " </rdf:RDF>\n",
                "</x:xmpmeta>\n",
                "<?xpacket end=\"w\"?>\n",
            )
        );
    }

    #[test]
    fn a_rating_outside_the_range_is_an_error() {
        assert!(write_rating(None, Some(6)).is_err());
        assert!(write_rating(None, Some(-2)).is_err());
    }
    fn labelled(source: &str, label: Option<&str>) -> String {
        String::from_utf8(write_label(Some(source.as_bytes()), label).unwrap()).unwrap()
    }

    fn with_attribute_label(label: &str) -> String {
        BRIDGE.replace(
            "   xmp:Rating=\"3\"\n",
            &format!("   xmp:Rating=\"3\"\n   xmp:Label=\"{label}\"\n"),
        )
    }

    fn with_element_label(label: &str) -> String {
        LIGHTROOM.replace(
            "   <xmp:Rating>2</xmp:Rating>\n",
            &format!("   <xmp:Rating>2</xmp:Rating>\n   <xmp:Label>{label}</xmp:Label>\n"),
        )
    }

    #[test]
    fn reads_an_attribute_label() {
        assert_eq!(
            read_label(with_attribute_label("Red").as_bytes()).unwrap(),
            Some("Red".to_string())
        );
    }

    #[test]
    fn reads_an_element_label() {
        assert_eq!(
            read_label(with_element_label("Blue").as_bytes()).unwrap(),
            Some("Blue".to_string())
        );
    }

    #[test]
    fn an_absent_or_empty_label_reads_as_none() {
        assert_eq!(read_label(BRIDGE.as_bytes()).unwrap(), None);
        assert_eq!(
            read_label(with_attribute_label("").as_bytes()).unwrap(),
            None
        );
        assert_eq!(read_label(with_element_label("").as_bytes()).unwrap(), None);
    }

    #[test]
    fn a_truncated_document_is_a_label_error() {
        let source = &BRIDGE[..BRIDGE.len() / 2];
        assert!(read_label(source.as_bytes()).is_err());
        assert!(write_label(Some(source.as_bytes()), Some("Red")).is_err());
    }

    #[test]
    fn sets_only_the_label_value() {
        let source = with_attribute_label("Red");
        assert_eq!(
            labelled(&source, Some("Green")),
            with_attribute_label("Green")
        );
        let source = with_element_label("Red");
        assert_eq!(
            labelled(&source, Some("Purple")),
            with_element_label("Purple")
        );
    }

    #[test]
    fn inserts_a_label_attribute_when_absent() {
        let out = labelled(NO_RATING, Some("Yellow"));
        assert_eq!(
            out,
            NO_RATING.replace(
                "xmp:CreatorTool=\"darktable\"/>",
                "xmp:CreatorTool=\"darktable\" xmp:Label=\"Yellow\"/>"
            )
        );
        assert_eq!(
            read_label(out.as_bytes()).unwrap(),
            Some("Yellow".to_string())
        );
    }

    #[test]
    fn clearing_removes_the_attribute() {
        assert_eq!(labelled(&with_attribute_label("Red"), None), BRIDGE);
        let inline = NO_RATING.replace(
            "xmp:CreatorTool=\"darktable\"/>",
            "xmp:CreatorTool=\"darktable\" xmp:Label=\"Red\"/>",
        );
        assert_eq!(labelled(&inline, None), NO_RATING);
    }

    #[test]
    fn clearing_removes_the_element_and_its_line() {
        assert_eq!(labelled(&with_element_label("Red"), None), LIGHTROOM);
        let shared = LIGHTROOM.replace(
            "<xmp:Rating>2</xmp:Rating>",
            "<xmp:Rating>2</xmp:Rating><xmp:Label>Red</xmp:Label>",
        );
        assert_eq!(labelled(&shared, None), LIGHTROOM);
    }

    #[test]
    fn clearing_without_a_label_is_a_no_op() {
        assert_eq!(labelled(BRIDGE, None), BRIDGE);
        assert_eq!(labelled(LIGHTROOM, None), LIGHTROOM);
    }

    #[test]
    fn a_fresh_label_template_and_no_sidecar_to_clear() {
        let bytes = write_label(None, Some("Red")).unwrap();
        assert_eq!(read_label(&bytes).unwrap(), Some("Red".to_string()));
        assert_eq!(read_rating(&bytes).unwrap(), None);
        assert!(write_label(None, None).is_err());
    }

    #[test]
    fn rating_and_label_keep_each_other() {
        let rated = patched(&with_element_label("Red"), Some(5));
        assert_eq!(
            read_label(rated.as_bytes()).unwrap(),
            Some("Red".to_string())
        );
        let both = labelled(&rated, Some("Blue"));
        assert_eq!(read_rating(both.as_bytes()).unwrap(), Some(5));
        assert_eq!(
            read_label(both.as_bytes()).unwrap(),
            Some("Blue".to_string())
        );

        let labelled_fresh = labelled(NO_RATING, Some("Green"));
        let rated = patched(&labelled_fresh, Some(2));
        assert_eq!(
            read_label(rated.as_bytes()).unwrap(),
            Some("Green".to_string())
        );
        assert_eq!(read_rating(rated.as_bytes()).unwrap(), Some(2));
    }

    #[test]
    fn an_element_label_with_an_entity_reference_reads_and_sets_whole() {
        let source = with_element_label("Red &amp; Blue");
        assert_eq!(
            read_label(source.as_bytes()).unwrap(),
            Some("Red &amp; Blue".to_string())
        );
        assert_eq!(
            labelled(&source, Some("Green")),
            with_element_label("Green")
        );
    }

    #[test]
    fn foreign_label_names_round_trip() {
        for label in ["Orange", "Rouge vif", "赤"] {
            let out = labelled(BRIDGE, Some(label));
            assert_eq!(read_label(out.as_bytes()).unwrap(), Some(label.to_string()));
            assert_eq!(labelled(&out, None), BRIDGE);
            let out = labelled(&with_element_label("Red"), Some(label));
            assert_eq!(out, with_element_label(label));
        }
    }
}
