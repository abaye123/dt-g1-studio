//! Street-name labels for the native map.
//!
//! The watch firmware does not label every street: it looks up the road near the
//! current GPS position and draws only that name. It does draw the names of
//! landuse polygons (that is how baked POI names appear, in Hebrew too). So for
//! each named street we inject a tiny closed polygon (a few meters, effectively a
//! dot) at a point on the street, carrying the street's name tags, in a custom
//! landuse class (`dtg1_label=street`) that [`LUT_ROW`] routes to the landuse
//! layer. Long streets get a label every [`SPACING_M`] meters.
//!
//! Runs on the downloaded OSM XML before compilation, after any language
//! rewrite, and copies all name tags so the compiler picks the same language.

use std::collections::HashMap;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;

/// Extra features.csv row for the label class. Landuse polygons are drawn up to
/// the 500 m display scale (above that the firmware blocks polygons), so LOD 500.
/// Hardware code 7203 (light green) is otherwise only used by a disabled row.
pub const LUT_ROW: &str =
    "7290;street_label;Light-green;500;landuse;dtg1_label=street;Street name label;7203;Light-green;500;1;";

/// Minimum distance between two labels of the same street name.
const SPACING_M: f64 = 250.0;
/// Side of the injected square, in meters.
const SIZE_M: f64 = 4.0;
/// Street classes that get labels.
const STREETS: &[&str] = &[
    "motorway", "trunk", "primary", "secondary", "tertiary", "unclassified",
    "residential", "living_street", "pedestrian", "service", "road",
];
/// Name tags copied onto the label (the compiler chooses among these).
const NAME_TAGS: &[&str] = &["name", "name:en", "int_name", "short_name", "short_name:en", "name:he"];

fn attr<'a>(line: &'a str, key: &str) -> Option<&'a str> {
    let pat = format!(" {key}=\"");
    let s = line.find(&pat)? + pat.len();
    let e = s + line[s..].find('"')?;
    Some(&line[s..e])
}

fn parse_tag(line: &str) -> Option<(String, String)> {
    let t = line.trim();
    if !t.starts_with("<tag ") {
        return None;
    }
    let k = attr(t, "k")?.to_string();
    let vs = t.find("v=\"")? + 3;
    let ve = vs + t[vs..].rfind('"')?;
    Some((k, t[vs..ve].to_string()))
}

struct Label {
    lat: f64,
    lon: f64,
    names: Vec<(String, String)>,
}

/// Approximate distance in meters (equirectangular, fine for a few km).
fn dist_m(a: (f64, f64), b: (f64, f64)) -> f64 {
    let k = 111_320.0;
    let dy = (a.0 - b.0) * k;
    let dx = (a.1 - b.1) * k * a.0.to_radians().cos();
    (dx * dx + dy * dy).sqrt()
}

/// Read the file once and decide where labels go.
fn plan(osm: &Path) -> Result<Vec<Label>, String> {
    let f = std::fs::File::open(osm).map_err(|e| e.to_string())?;
    let mut nodes: HashMap<i64, (f64, f64)> = HashMap::new();
    let mut placed: HashMap<String, Vec<(f64, f64)>> = HashMap::new();
    let mut labels = Vec::new();
    let mut in_way = false;
    let (mut refs, mut tags): (Vec<i64>, Vec<(String, String)>) = (Vec::new(), Vec::new());

    for line in BufReader::new(f).lines() {
        let line = line.map_err(|e| e.to_string())?;
        let t = line.trim_start();
        if t.starts_with("<node ") {
            if let (Some(id), Some(lat), Some(lon)) = (attr(t, "id"), attr(t, "lat"), attr(t, "lon")) {
                if let (Ok(id), Ok(lat), Ok(lon)) = (id.parse(), lat.parse(), lon.parse()) {
                    nodes.insert(id, (lat, lon));
                }
            }
        } else if t.starts_with("<way ") && !t.trim_end().ends_with("/>") {
            in_way = true;
            refs.clear();
            tags.clear();
        } else if in_way && t.starts_with("<nd ") {
            if let Some(r) = attr(t, "ref").and_then(|r| r.parse().ok()) {
                refs.push(r);
            }
        } else if in_way && t.starts_with("<tag ") {
            if let Some(kv) = parse_tag(t) {
                tags.push(kv);
            }
        } else if in_way && t.starts_with("</way>") {
            in_way = false;
            let is_street = tags.iter().any(|(k, v)| k == "highway" && STREETS.contains(&v.as_str()));
            let name = tags.iter().find(|(k, _)| k == "name").map(|(_, v)| v.clone());
            let (Some(name), true) = (name, is_street) else { continue };
            let pts: Vec<(f64, f64)> = refs.iter().filter_map(|r| nodes.get(r).copied()).collect();
            if pts.is_empty() {
                continue;
            }
            let mid = pts[pts.len() / 2];
            let seen = placed.entry(name).or_default();
            if seen.iter().any(|p| dist_m(*p, mid) < SPACING_M) {
                continue;
            }
            seen.push(mid);
            labels.push(Label {
                lat: mid.0,
                lon: mid.1,
                names: tags.iter().filter(|(k, _)| NAME_TAGS.contains(&k.as_str())).cloned().collect(),
            });
        }
    }
    Ok(labels)
}

/// Inject street-name label polygons into an OSM XML file in place. Returns the
/// number of labels added. New nodes go before the first way (the compiler reads
/// nodes first), new ways before `</osm>`. Ids are negative, as in JOSM edits.
pub fn add_street_labels(osm: &Path) -> Result<usize, String> {
    let labels = plan(osm)?;
    if labels.is_empty() {
        return Ok(0);
    }
    let mut new_nodes = String::new();
    let mut new_ways = String::new();
    let mut next_id: i64 = -1;
    let half = SIZE_M / 2.0;
    for l in &labels {
        let dlat = half / 111_320.0;
        let dlon = half / (111_320.0 * l.lat.to_radians().cos());
        let corners = [(l.lat + dlat, l.lon - dlon), (l.lat + dlat, l.lon + dlon), (l.lat - dlat, l.lon + dlon), (l.lat - dlat, l.lon - dlon)];
        let mut ids = Vec::new();
        for (lat, lon) in corners {
            new_nodes.push_str(&format!("  <node id=\"{next_id}\" lat=\"{lat:.7}\" lon=\"{lon:.7}\"/>\n"));
            ids.push(next_id);
            next_id -= 1;
        }
        new_ways.push_str(&format!("  <way id=\"{next_id}\">\n"));
        next_id -= 1;
        for id in ids.iter().chain(std::iter::once(&ids[0])) {
            new_ways.push_str(&format!("    <nd ref=\"{id}\"/>\n"));
        }
        new_ways.push_str("    <tag k=\"dtg1_label\" v=\"street\"/>\n");
        for (k, v) in &l.names {
            new_ways.push_str(&format!("    <tag k=\"{k}\" v=\"{v}\"/>\n"));
        }
        new_ways.push_str("  </way>\n");
    }

    let tmp = osm.with_extension("osm.lbl");
    {
        let input = std::fs::File::open(osm).map_err(|e| e.to_string())?;
        let mut w = BufWriter::new(std::fs::File::create(&tmp).map_err(|e| e.to_string())?);
        let mut nodes_written = false;
        let mut ways_written = false;
        for line in BufReader::new(input).lines() {
            let line = line.map_err(|e| e.to_string())?;
            let t = line.trim_start();
            if !nodes_written && (t.starts_with("<way ") || t.starts_with("<relation ") || t.starts_with("</osm>")) {
                w.write_all(new_nodes.as_bytes()).map_err(|e| e.to_string())?;
                nodes_written = true;
            }
            if !ways_written && (t.starts_with("<relation ") || t.starts_with("</osm>")) {
                w.write_all(new_ways.as_bytes()).map_err(|e| e.to_string())?;
                ways_written = true;
            }
            writeln!(w, "{line}").map_err(|e| e.to_string())?;
        }
        if !ways_written {
            return Err("osm file has no closing </osm>".into());
        }
        w.flush().map_err(|e| e.to_string())?;
    }
    std::fs::rename(&tmp, osm).map_err(|e| e.to_string())?;
    Ok(labels.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn labels_streets_once_per_spacing() {
        // A street split into two ways ~110 m apart (same name -> one label),
        // a far way of the same street (~1.1 km -> second label), an unnamed
        // street and a footway (no labels).
        let src = r#"<?xml version="1.0" encoding="UTF-8"?>
<osm version="0.6">
 <node id="1" lat="31.8000" lon="35.2000"/>
 <node id="2" lat="31.8010" lon="35.2000"/>
 <node id="3" lat="31.8020" lon="35.2000"/>
 <node id="4" lat="31.8100" lon="35.2000"/>
 <node id="5" lat="31.8110" lon="35.2000"/>
 <way id="10">
  <nd ref="1"/>
  <nd ref="2"/>
  <tag k="highway" v="residential"/>
  <tag k="name" v="גולדה מאיר"/>
  <tag k="name:en" v="Golda Meir"/>
 </way>
 <way id="11">
  <nd ref="2"/>
  <nd ref="3"/>
  <tag k="highway" v="residential"/>
  <tag k="name" v="גולדה מאיר"/>
 </way>
 <way id="12">
  <nd ref="4"/>
  <nd ref="5"/>
  <tag k="highway" v="residential"/>
  <tag k="name" v="גולדה מאיר"/>
 </way>
 <way id="13">
  <nd ref="1"/>
  <nd ref="3"/>
  <tag k="highway" v="residential"/>
 </way>
 <way id="14">
  <nd ref="1"/>
  <nd ref="3"/>
  <tag k="highway" v="footway"/>
  <tag k="name" v="שביל"/>
 </way>
</osm>
"#;
        let p = std::env::temp_dir().join(format!("dtg1_lbl_{}.osm", std::process::id()));
        std::fs::write(&p, src).unwrap();
        let n = add_street_labels(&p).unwrap();
        let out = std::fs::read_to_string(&p).unwrap();
        std::fs::remove_file(&p).ok();
        assert_eq!(n, 2);
        assert_eq!(out.matches("dtg1_label").count(), 2);
        // the footway is not labelled: its name appears only on the original way
        assert_eq!(out.matches("שביל").count(), 1);
        // new nodes come before the first original way, new ways before </osm>
        let first_way = out.find("<way id=\"10\"").unwrap();
        assert!(out.find("<node id=\"-1\"").unwrap() < first_way);
        assert!(out.rfind("dtg1_label").unwrap() < out.find("</osm>").unwrap());
        // name tags copied, including name:en
        assert!(out.contains(r#"<tag k="name:en" v="Golda Meir"/>"#));
        // each label polygon is closed (first ref repeated)
        assert_eq!(out.matches("<nd ref=\"-1\"/>").count(), 2);
    }
}
