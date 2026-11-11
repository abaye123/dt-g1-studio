//! Choose the label language of a compiled map by rewriting the OSM data before
//! compilation.
//!
//! The bundled compiler picks a feature's label in this order:
//! `short_name:en`, `int_name`, `name:en`, `short_name`, `name`, so English wins
//! whenever it exists. We cannot change the compiler binary, but we control the
//! `map.osm` it reads. For Hebrew we drop the English name tags and, when the
//! element has `name:he`, make it the `name` (and drop `short_name`, which comes
//! earlier in the order and may be in another language).
//!
//! The OSM API and Overpass both emit one `<tag .../>` per line inside an element,
//! which lets us rewrite element by element with a simple line scan.

use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;

/// Tags the compiler prefers over `name`; removed for Hebrew labels.
const EN_TAGS: &[&str] = &["short_name:en", "int_name", "name:en"];

/// Extract the `k` and `v` attributes of a `<tag k=".." v=".."/>` line. The
/// values stay XML-escaped, exactly as in the file.
fn parse_tag(line: &str) -> Option<(String, String)> {
    let t = line.trim();
    if !t.starts_with("<tag ") {
        return None;
    }
    let k_start = t.find("k=\"")? + 3;
    let k_end = k_start + t[k_start..].find('"')?;
    let v_start = t.find("v=\"")? + 3;
    let v_end = v_start + t[v_start..].rfind('"')?;
    Some((t[k_start..k_end].to_string(), t[v_start..v_end].to_string()))
}

fn indent_of(line: &str) -> &str {
    &line[..line.len() - line.trim_start().len()]
}

/// Rewrite the inner lines of one element for Hebrew labels.
fn localize_he(inner: Vec<String>) -> Vec<String> {
    let mut he: Option<String> = None;
    let mut has_name = false;
    let mut tag_indent = String::from("    ");
    for l in &inner {
        if let Some((k, v)) = parse_tag(l) {
            tag_indent = indent_of(l).to_string();
            match k.as_str() {
                "name:he" => he = Some(v),
                "name" => has_name = true,
                _ => {}
            }
        }
    }
    let mut out = Vec::with_capacity(inner.len() + 1);
    for l in inner {
        match parse_tag(&l) {
            Some((k, _)) if EN_TAGS.contains(&k.as_str()) => continue,
            Some((k, _)) if k == "short_name" && he.is_some() => continue,
            Some((k, _)) if k == "name" => match &he {
                Some(h) => out.push(format!("{}<tag k=\"name\" v=\"{}\"/>", indent_of(&l), h)),
                None => out.push(l),
            },
            _ => out.push(l),
        }
    }
    if !has_name {
        if let Some(h) = he {
            out.push(format!("{tag_indent}<tag k=\"name\" v=\"{h}\"/>"));
        }
    }
    out
}

fn opens_element(t: &str) -> Option<&'static str> {
    for (open, close) in [("<node ", "</node>"), ("<way ", "</way>"), ("<relation ", "</relation>")] {
        if t.starts_with(open) && !t.ends_with("/>") {
            return Some(close);
        }
    }
    None
}

/// Rewrite an OSM XML file in place so the compiler picks Hebrew labels.
pub fn localize_hebrew(osm: &Path) -> Result<(), String> {
    let tmp = osm.with_extension("osm.tmp");
    {
        let input = std::fs::File::open(osm).map_err(|e| e.to_string())?;
        let output = std::fs::File::create(&tmp).map_err(|e| e.to_string())?;
        let mut w = BufWriter::new(output);
        let mut close: Option<&'static str> = None;
        let mut inner: Vec<String> = Vec::new();
        for line in BufReader::new(input).lines() {
            let line = line.map_err(|e| e.to_string())?;
            let t = line.trim();
            match close {
                Some(c) if t.starts_with(c) => {
                    for l in localize_he(std::mem::take(&mut inner)) {
                        writeln!(w, "{l}").map_err(|e| e.to_string())?;
                    }
                    writeln!(w, "{line}").map_err(|e| e.to_string())?;
                    close = None;
                }
                Some(_) => inner.push(line),
                None => {
                    close = opens_element(t);
                    writeln!(w, "{line}").map_err(|e| e.to_string())?;
                }
            }
        }
        // A truncated file: write whatever was buffered unchanged.
        for l in inner {
            writeln!(w, "{l}").map_err(|e| e.to_string())?;
        }
        w.flush().map_err(|e| e.to_string())?;
    }
    std::fs::rename(&tmp, osm).map_err(|e| e.to_string())
}

#[cfg(test)]
mod itest {
    use super::*;

    // Real-data check: download a small area in Ramot, compile it in both label
    // modes with the bundled compiler, and compare the names in roads.db.
    //   cargo test -p dt-g1-studio-app --lib hebrew_labels_real -- --ignored --nocapture
    #[test]
    #[ignore]
    fn hebrew_labels_real() {
        let base = std::env::temp_dir().join("dtg1_osmlang_itest");
        let _ = std::fs::remove_dir_all(&base);
        let bundled = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("resources").join("map-compiler");
        let osm_en = base.join("en.osm");
        crate::mapcompile::download_osm(35.186, 31.817, 35.196, 31.824, &osm_en).expect("download");
        let osm_he = base.join("he.osm");
        std::fs::copy(&osm_en, &osm_he).unwrap();
        localize_hebrew(&osm_he).expect("localize");
        let osm_lbl = base.join("he_labels.osm");
        std::fs::copy(&osm_he, &osm_lbl).unwrap();
        let n_labels = crate::osmlabels::add_street_labels(&osm_lbl).expect("labels");
        println!("street labels injected: {n_labels}");

        let none: &[&str] = &[];
        let lbl_rows: &[&str] = &[crate::osmlabels::LUT_ROW];
        for (tag, osm, style, rows) in [
            ("en", &osm_en, "bold", none),
            ("he", &osm_he, "bold", none),
            ("he_factory", &osm_he, "factory", none),
            ("he_labels", &osm_lbl, "factory", lbl_rows),
        ] {
            let comp = base.join(format!("compiler_{tag}"));
            std::fs::create_dir_all(&comp).unwrap();
            for e in std::fs::read_dir(&bundled).unwrap().flatten() {
                std::fs::copy(e.path(), comp.join(e.file_name())).unwrap();
            }
            let staging = base.join(format!("out_{tag}"));
            let files = crate::mapcompile::compile(&comp, osm, &staging, "Test", "landuse", style, rows).expect("compile");
            let db = std::fs::read(staging.join("roads.db")).unwrap_or_default();
            // header MD5 must still match the body after the space restore
            use md5::{Digest, Md5};
            assert_eq!(&db[16..32], Md5::digest(&db[32..]).as_slice(), "roads.db md5");
            let text = String::from_utf8_lossy(&db);
            let hebrew = text.chars().filter(|c| ('\u{05D0}'..='\u{05EA}').contains(c)).count();
            let latin = text.chars().filter(|c| c.is_ascii_alphabetic()).count();
            println!("[{tag}] files={:?}\n  roads.db hebrew letters={hebrew} latin letters={latin}", files.iter().map(|(n, _)| n).collect::<Vec<_>>());
            let names: Vec<String> = text
                .split('\0')
                .map(|s| s.trim().to_string())
                .filter(|s| s.chars().count() > 2 && s.chars().any(|c| c.is_alphabetic()))
                .take(8)
                .collect();
            println!("  sample: {names:?}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<osm version="0.6">
 <node id="1" lat="31.8" lon="35.2">
  <tag k="amenity" v="pharmacy"/>
  <tag k="name" v="Super-Pharm"/>
  <tag k="name:en" v="Super-Pharm"/>
  <tag k="name:he" v="סופר-פארם"/>
 </node>
 <node id="2" lat="31.8" lon="35.2"/>
 <way id="10">
  <nd ref="1"/>
  <tag k="highway" v="residential"/>
  <tag k="name" v="גולדה מאיר"/>
  <tag k="name:en" v="Golda Meir"/>
  <tag k="int_name" v="Golda Meir"/>
 </way>
 <way id="11">
  <nd ref="1"/>
  <tag k="highway" v="primary"/>
  <tag k="name:he" v="שדרות &quot;הרצל&quot;"/>
  <tag k="short_name" v="Herzl"/>
 </way>
</osm>
"#;

    fn run(src: &str) -> String {
        let p = std::env::temp_dir().join(format!("dtg1_osmlang_{}.osm", std::process::id()));
        std::fs::write(&p, src).unwrap();
        localize_hebrew(&p).unwrap();
        let out = std::fs::read_to_string(&p).unwrap();
        std::fs::remove_file(&p).ok();
        out
    }

    #[test]
    fn prefers_hebrew_and_drops_english() {
        let out = run(SAMPLE);
        // name:he replaces name
        assert!(out.contains(r#"<tag k="name" v="סופר-פארם"/>"#));
        assert!(!out.contains(r#"<tag k="name" v="Super-Pharm"/>"#));
        // English preferences removed
        assert!(!out.contains("name:en"));
        assert!(!out.contains("int_name"));
        // plain Hebrew name kept when there is no name:he
        assert!(out.contains(r#"<tag k="name" v="גולדה מאיר"/>"#));
        // name added from name:he when missing; short_name dropped; escapes kept
        assert!(out.contains(r#"<tag k="name" v="שדרות &quot;הרצל&quot;"/>"#));
        assert!(!out.contains("short_name"));
        // structure untouched
        assert!(out.contains(r#"<node id="2" lat="31.8" lon="35.2"/>"#));
        assert!(out.contains(r#"<nd ref="1"/>"#));
        assert_eq!(out.matches("</way>").count(), 2);
    }
}
