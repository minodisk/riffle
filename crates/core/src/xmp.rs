//! XMP sidecar reading and writing for ratings.
//!
//! A rating is a single signed integer in `-1..=5`: `0` is unrated, `1`-`5`
//! are stars and `-1` is a reject, which is the convention Adobe Bridge
//! writes and darktable reads. Nothing but `xmp:Rating` is ever written.
//!
//! An existing sidecar is patched by splicing the bytes of the `Rating`
//! value, so a Lightroom sidecar keeps its `crs:` develop settings
//! byte-for-byte; a sidecar is never regenerated from a parse.

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
    match locate(text)? {
        Location::Value { start, end } => Ok(text[start..end]
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
        return Ok(template(value).into_bytes());
    };
    let text = std::str::from_utf8(existing).map_err(|e| format!("not UTF-8: {e}"))?;
    let mut out = String::with_capacity(text.len() + 32);
    match locate(text)? {
        Location::Value { start, end } => {
            out.push_str(&text[..start]);
            out.push_str(&value.to_string());
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
            out.push_str(&format!(" {prefix}:Rating=\"{value}\""));
            out.push_str(&text[at..]);
        }
    }
    Ok(out.into_bytes())
}

fn template(value: i8) -> String {
    format!(
        "<?xpacket begin=\"\" id=\"W5M0MpCehiHzreSzNTczkc9d\"?>\n\
         <x:xmpmeta xmlns:x=\"adobe:ns:meta/\" x:xmptk=\"riffle\">\n\
         \x20<rdf:RDF xmlns:rdf=\"{RDF_NS}\">\n\
         \x20 <rdf:Description rdf:about=\"\" xmlns:xmp=\"{XMP_NS}\" xmp:Rating=\"{value}\"/>\n\
         \x20</rdf:RDF>\n\
         </x:xmpmeta>\n\
         <?xpacket end=\"w\"?>\n"
    )
}

enum Location {
    /// The byte range of the `Rating` value, attribute or element text.
    Value { start: usize, end: usize },
    /// Where to insert an attribute in the first `rdf:Description` start tag.
    Insert {
        at: usize,
        prefix: String,
        declare: bool,
    },
}

fn locate(text: &str) -> Result<Location, String> {
    let mut reader = NsReader::from_str(text);
    let mut pos = 0usize;
    let mut insert: Option<Location> = None;
    let mut first_description_seen = false;
    let mut in_rating = false;
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
                        if bound_to(&ans, XMP_NS) && alocal.as_ref() == "Rating" {
                            if let Some((s, e)) = attr_value_range(span, attr.key.as_ref()) {
                                return Ok(Location::Value {
                                    start: pos + s,
                                    end: pos + e,
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
                } else if bound_to(&ns, XMP_NS) && local.as_ref() == "Rating" {
                    in_rating = matches!(event, Event::Start(_));
                }
            }
            Event::Text(_) if in_rating => {
                return Ok(Location::Value { start: pos, end });
            }
            Event::End(e) => {
                let (ns, local) = reader.resolver().resolve_element(e.name());
                if bound_to(&ns, XMP_NS) && local.as_ref() == "Rating" && in_rating {
                    // An empty element-form property, `<xmp:Rating></xmp:Rating>`,
                    // never produces a `Text` event: splice the empty range
                    // between the tags instead of falling through to `insert`.
                    return Ok(Location::Value {
                        start: pos,
                        end: pos,
                    });
                }
                in_rating = false;
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

/// The prefix to write `Rating` with, and whether it has to be declared.
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

/// The byte range of `key`'s value inside a start tag, quotes excluded.
///
/// quick-xml reports positions per event, not per attribute, so the value is
/// located by scanning the start tag's own range, which is exact.
fn attr_value_range(tag: &str, key: &str) -> Option<(usize, usize)> {
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
        return Some((value_start, value_end));
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
}
