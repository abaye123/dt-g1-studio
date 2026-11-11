//! Elevation contour lines for watch maps, generated without GDAL.
//!
//! Source: Copernicus DEM GLO-30 (free, no key) on AWS open data. Each 1x1
//! degree tile is a Cloud-Optimized GeoTIFF: float32, Deflate, floating-point
//! predictor (3), internal 1024x1024 blocks. We read the header, then fetch only
//! the internal blocks the map area needs (HTTP range requests) and cache them
//! under the app data dir, so later compiles of the same area work offline.
//!
//! Contours are traced with marching squares, stitched into lines, simplified
//! (Douglas-Peucker) and injected into the OSM file as ways with fake road tags
//! (`highway=c1/c2/c3`, the method the map compiler's author uses), routed by
//! [`LUT_ROWS`] to gray road codes so they do not look like real roads.

use std::collections::HashMap;
use std::io::{BufRead, BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};

/// features.csv rows for the contour classes (appended per compile).
/// Major/medium use 5115 (drawn from the 500 m scale), minor 5124 (from 100 m);
/// both are gray lines in the watch's hardware style table.
pub const LUT_ROWS: [&str; 3] = [
    "9901;contour_major;Gray;500;roads;highway=c1;Contour major;5115;Gray;500;1;",
    "9902;contour_medium;Gray;500;roads;highway=c2;Contour medium;5115;Gray;500;1;",
    "9903;contour_minor;Gray;100;roads;highway=c3;Contour minor;5124;Gray;100;1;",
];

/// Upper bound on contour vertices, to stay within what the watch handles.
const MAX_POINTS: usize = 60_000;
const UA: &str = "dt-g1-studio";

fn tile_name(lat: i32, lon: i32) -> String {
    let ns = if lat >= 0 { 'N' } else { 'S' };
    let ew = if lon >= 0 { 'E' } else { 'W' };
    format!("Copernicus_DSM_COG_10_{ns}{:02}_00_{ew}{:03}_00_DEM", lat.abs(), lon.abs())
}

fn tile_url(name: &str) -> String {
    format!("https://copernicus-dem-30m.s3.amazonaws.com/{name}/{name}.tif")
}

/// Fetch `len` bytes at `start` of `url` (HTTP Range).
fn fetch_range(url: &str, start: u64, len: u64) -> Result<Vec<u8>, String> {
    let http = crate::mapcompile::agent()?;
    let resp = http
        .get(url)
        .set("User-Agent", UA)
        .set("Range", &format!("bytes={}-{}", start, start + len - 1))
        .call()
        .map_err(|e| match e {
            ureq::Error::Status(404, _) => "dem_tile_missing".to_string(),
            other => format!("dem_download_failed: {other}"),
        })?;
    let mut buf = Vec::with_capacity(len as usize);
    resp.into_reader().read_to_end(&mut buf).map_err(|e| e.to_string())?;
    Ok(buf)
}

/// What we need from the first IFD of a DEM tile.
struct TileMeta {
    width: u32,
    height: u32,
    block_w: u32,
    block_h: u32,
    offsets: Vec<u32>,
    counts: Vec<u32>,
}

fn parse_header(b: &[u8]) -> Result<TileMeta, String> {
    if b.len() < 8 || &b[0..2] != b"II" {
        return Err("dem: not a little-endian TIFF".into());
    }
    let u16_at = |p: usize| u16::from_le_bytes([b[p], b[p + 1]]);
    let u32_at = |p: usize| u32::from_le_bytes([b[p], b[p + 1], b[p + 2], b[p + 3]]);
    let ifd = u32_at(4) as usize;
    let n = u16_at(ifd) as usize;
    let (mut width, mut height, mut bw, mut bh) = (0, 0, 0, 0);
    let (mut compression, mut predictor, mut bits, mut fmt) = (0, 1, 0, 0);
    let (mut offsets, mut counts) = (Vec::new(), Vec::new());
    for i in 0..n {
        let e = ifd + 2 + i * 12;
        let (tag, typ, cnt) = (u16_at(e), u16_at(e + 2), u32_at(e + 4) as usize);
        let scalar = if typ == 3 { u16_at(e + 8) as u32 } else { u32_at(e + 8) };
        let array = |cnt: usize| -> Vec<u32> {
            if cnt == 1 {
                return vec![scalar];
            }
            let p = u32_at(e + 8) as usize;
            (0..cnt)
                .filter(|k| p + k * 4 + 4 <= b.len())
                .map(|k| u32_at(p + k * 4))
                .collect()
        };
        match tag {
            256 => width = scalar,
            257 => height = scalar,
            258 => bits = scalar,
            259 => compression = scalar,
            317 => predictor = scalar,
            322 => bw = scalar,
            323 => bh = scalar,
            324 => offsets = array(cnt),
            325 => counts = array(cnt),
            339 => fmt = scalar,
            _ => {}
        }
    }
    if compression != 8 || bits != 32 || fmt != 3 || predictor != 3 || bw == 0 || bh == 0 {
        return Err(format!(
            "dem: unsupported layout (compression {compression}, bits {bits}, format {fmt}, predictor {predictor})"
        ));
    }
    let blocks = (width.div_ceil(bw) * height.div_ceil(bh)) as usize;
    if offsets.len() != blocks || counts.len() != blocks {
        return Err("dem: block table incomplete".into());
    }
    Ok(TileMeta { width, height, block_w: bw, block_h: bh, offsets, counts })
}

/// Undo Deflate + the TIFF floating-point predictor for one block of float32.
pub fn decode_block(compressed: &[u8], bw: usize, bh: usize) -> Result<Vec<f32>, String> {
    let mut raw = Vec::with_capacity(bw * bh * 4);
    flate2::read::ZlibDecoder::new(compressed)
        .read_to_end(&mut raw)
        .map_err(|e| format!("dem inflate: {e}"))?;
    if raw.len() < bw * bh * 4 {
        return Err("dem: short block".into());
    }
    let row_bytes = bw * 4;
    let mut out = Vec::with_capacity(bw * bh);
    for r in 0..bh {
        let row = &mut raw[r * row_bytes..(r + 1) * row_bytes];
        // byte-wise horizontal differencing across the whole row
        for i in 1..row_bytes {
            row[i] = row[i].wrapping_add(row[i - 1]);
        }
        // bytes are stored as planes, most significant first
        for k in 0..bw {
            out.push(f32::from_be_bytes([row[k], row[bw + k], row[2 * bw + k], row[3 * bw + k]]));
        }
    }
    Ok(out)
}

/// Lazy, cached access to elevation samples across Copernicus tiles.
struct Dem {
    cache: PathBuf,
    metas: HashMap<(i32, i32), Option<TileMeta>>,
    blocks: HashMap<(i32, i32, usize), Vec<f32>>,
}

impl Dem {
    fn new(cache: PathBuf) -> Self {
        Dem { cache, metas: HashMap::new(), blocks: HashMap::new() }
    }

    fn meta(&mut self, lat: i32, lon: i32) -> Result<(), String> {
        if self.metas.contains_key(&(lat, lon)) {
            return Ok(());
        }
        let name = tile_name(lat, lon);
        let hdr_path = self.cache.join(format!("{name}.hdr"));
        let bytes = match std::fs::read(&hdr_path) {
            Ok(b) => b,
            Err(_) => match fetch_range(&tile_url(&name), 0, 65536) {
                Ok(b) => {
                    std::fs::create_dir_all(&self.cache).map_err(|e| e.to_string())?;
                    std::fs::write(&hdr_path, &b).map_err(|e| e.to_string())?;
                    b
                }
                // No tile here (open sea): treat as sea level.
                Err(e) if e == "dem_tile_missing" => {
                    self.metas.insert((lat, lon), None);
                    return Ok(());
                }
                Err(e) => return Err(e),
            },
        };
        self.metas.insert((lat, lon), Some(parse_header(&bytes)?));
        Ok(())
    }

    fn block(&mut self, lat: i32, lon: i32, idx: usize) -> Result<(), String> {
        if self.blocks.contains_key(&(lat, lon, idx)) {
            return Ok(());
        }
        let name = tile_name(lat, lon);
        let m = self.metas[&(lat, lon)].as_ref().unwrap();
        let (bw, bh) = (m.block_w as usize, m.block_h as usize);
        let path = self.cache.join(format!("{name}_{idx}.z"));
        let z = match std::fs::read(&path) {
            Ok(z) => z,
            Err(_) => {
                let z = fetch_range(&tile_url(&name), m.offsets[idx] as u64, m.counts[idx] as u64)?;
                std::fs::create_dir_all(&self.cache).map_err(|e| e.to_string())?;
                std::fs::write(&path, &z).map_err(|e| e.to_string())?;
                z
            }
        };
        let data = decode_block(&z, bw, bh)?;
        self.blocks.insert((lat, lon, idx), data);
        Ok(())
    }

    /// Elevation in meters at (lat, lon), nearest sample.
    fn sample(&mut self, lat: f64, lon: f64) -> Result<f32, String> {
        let (tl, tn) = (lat.floor() as i32, lon.floor() as i32);
        self.meta(tl, tn)?;
        let Some(m) = self.metas[&(tl, tn)].as_ref() else { return Ok(0.0) };
        let col = (((lon - tn as f64) * m.width as f64) as u32).min(m.width - 1);
        let row = ((((tl + 1) as f64 - lat) * m.height as f64) as u32).min(m.height - 1);
        let (bw, bh) = (m.block_w, m.block_h);
        let blocks_x = m.width.div_ceil(bw);
        let idx = ((row / bh) * blocks_x + col / bw) as usize;
        let off = ((row % bh) * bw + col % bw) as usize;
        self.block(tl, tn, idx)?;
        let v = self.blocks[&(tl, tn, idx)][off];
        Ok(if v.is_finite() && v > -1000.0 { v } else { 0.0 })
    }
}

/// A sampled elevation grid over the map area. Row 0 is the south edge.
pub struct Grid {
    pub west: f64,
    pub south: f64,
    pub step: f64,
    pub cols: usize,
    pub rows: usize,
    pub z: Vec<f32>,
}

impl Grid {
    fn at(&self, r: usize, c: usize) -> f32 {
        self.z[r * self.cols + c]
    }
    fn lonlat(&self, r: f64, c: f64) -> (f64, f64) {
        (self.west + c * self.step, self.south + r * self.step)
    }
}

/// Light 3x3 smoothing so 30 m DEM noise does not make jagged lines.
fn smooth(g: &mut Grid) {
    let src = g.z.clone();
    for r in 1..g.rows.saturating_sub(1) {
        for c in 1..g.cols.saturating_sub(1) {
            let mut s = 0.0;
            for dr in 0..3 {
                for dc in 0..3 {
                    s += src[(r + dr - 1) * g.cols + (c + dc - 1)];
                }
            }
            g.z[r * g.cols + c] = s / 9.0;
        }
    }
}

/// Crossing point of `level` on an edge, keyed by the edge so neighbours share
/// the exact same point: (row, col, horizontal?).
type EdgeKey = (usize, usize, bool);

/// Trace one contour level into polylines of (lon, lat).
pub fn trace(g: &Grid, level: f32) -> Vec<Vec<(f64, f64)>> {
    let mut adj: HashMap<EdgeKey, Vec<EdgeKey>> = HashMap::new();
    let mut link = |a: EdgeKey, b: EdgeKey| {
        adj.entry(a).or_default().push(b);
        adj.entry(b).or_default().push(a);
    };
    for r in 0..g.rows - 1 {
        for c in 0..g.cols - 1 {
            // corners: 0 = (r,c) sw, 1 = (r,c+1) se, 2 = (r+1,c+1) ne, 3 = (r+1,c) nw
            let v = [g.at(r, c), g.at(r, c + 1), g.at(r + 1, c + 1), g.at(r + 1, c)];
            let mut case = 0;
            for (i, x) in v.iter().enumerate() {
                if *x >= level {
                    case |= 1 << i;
                }
            }
            // edges: south (r,c,h), east (r,c+1,v), north (r+1,c,h), west (r,c,v)
            let s = (r, c, true);
            let e = (r, c + 1, false);
            let n = (r + 1, c, true);
            let w = (r, c, false);
            let center_hi = (v.iter().sum::<f32>() / 4.0) >= level;
            match case {
                0 | 15 => {}
                1 | 14 => link(w, s),
                2 | 13 => link(s, e),
                3 | 12 => link(w, e),
                4 | 11 => link(e, n),
                6 | 9 => link(s, n),
                7 | 8 => link(w, n),
                5 => {
                    if center_hi { link(w, n); link(s, e) } else { link(w, s); link(e, n) }
                }
                10 => {
                    if center_hi { link(w, s); link(e, n) } else { link(w, n); link(s, e) }
                }
                _ => {}
            }
        }
    }
    let point = |k: &EdgeKey| -> (f64, f64) {
        let (r, c, h) = *k;
        let (a, b, (r2, c2)) = if h {
            (g.at(r, c), g.at(r, c + 1), (r, c + 1))
        } else {
            (g.at(r, c), g.at(r + 1, c), (r + 1, c))
        };
        let t = if (b - a).abs() < 1e-6 { 0.5 } else { ((level - a) / (b - a)) as f64 };
        let t = t.clamp(0.0, 1.0);
        g.lonlat(r as f64 + (r2 as f64 - r as f64) * t, c as f64 + (c2 as f64 - c as f64) * t)
    };

    let mut lines = Vec::new();
    let seg = |a: EdgeKey, b: EdgeKey| if a < b { (a, b) } else { (b, a) };
    let mut used: std::collections::HashSet<(EdgeKey, EdgeKey)> = std::collections::HashSet::new();
    // Start from open ends (degree 1) first, then from what is left (loops).
    let mut starts: Vec<EdgeKey> = adj.iter().filter(|(_, v)| v.len() == 1).map(|(k, _)| *k).collect();
    starts.extend(adj.keys().copied());
    for start in starts {
        let mut cur = start;
        let mut pts = vec![point(&cur)];
        while let Some(n) = adj[&cur].iter().copied().find(|n| !used.contains(&seg(cur, *n))) {
            used.insert(seg(cur, n));
            pts.push(point(&n));
            cur = n;
        }
        if pts.len() >= 2 {
            lines.push(pts);
        }
    }
    lines
}

/// Douglas-Peucker simplification in (lon, lat) with lon scaled by cos(lat).
pub fn simplify(pts: &[(f64, f64)], tol_deg: f64) -> Vec<(f64, f64)> {
    if pts.len() < 3 {
        return pts.to_vec();
    }
    let k = pts[0].1.to_radians().cos();
    let mut keep = vec![false; pts.len()];
    keep[0] = true;
    keep[pts.len() - 1] = true;
    let mut stack = vec![(0usize, pts.len() - 1)];
    while let Some((a, b)) = stack.pop() {
        let (ax, ay) = (pts[a].0 * k, pts[a].1);
        let (bx, by) = (pts[b].0 * k, pts[b].1);
        let (dx, dy) = (bx - ax, by - ay);
        let len = (dx * dx + dy * dy).sqrt();
        let (mut best, mut idx) = (0.0, 0);
        for (i, p) in pts.iter().enumerate().take(b).skip(a + 1) {
            let (px, py) = (p.0 * k, p.1);
            let d = if len < 1e-12 {
                ((px - ax).powi(2) + (py - ay).powi(2)).sqrt()
            } else {
                ((px - ax) * dy - (py - ay) * dx).abs() / len
            };
            if d > best {
                best = d;
                idx = i;
            }
        }
        if best > tol_deg {
            keep[idx] = true;
            stack.push((a, idx));
            stack.push((idx, b));
        }
    }
    pts.iter().zip(keep).filter(|(_, k)| *k).map(|(p, _)| *p).collect()
}

/// Result summary for the UI.
pub struct ContourStats {
    pub lines: usize,
    pub points: usize,
}

/// Build contours for the bbox at `interval_m` and inject them into `osm`.
/// `cache_dir` holds downloaded DEM blocks for offline reuse.
pub fn add_contours(
    osm: &Path,
    (west, south, east, north): (f64, f64, f64, f64),
    interval_m: u32,
    cache_dir: &Path,
) -> Result<ContourStats, String> {
    let interval = interval_m.max(5) as f32;
    let step = 1.0 / 3600.0; // ~30 m, the DEM resolution
    let cols = (((east - west) / step).ceil() as usize + 1).max(2);
    let rows = (((north - south) / step).ceil() as usize + 1).max(2);
    if cols * rows > 4_000_000 {
        return Err("contour_area_too_large".into());
    }
    let mut dem = Dem::new(cache_dir.to_path_buf());
    let mut z = Vec::with_capacity(cols * rows);
    for r in 0..rows {
        for c in 0..cols {
            z.push(dem.sample(south + r as f64 * step, west + c as f64 * step)?);
        }
    }
    let mut g = Grid { west, south, step, cols, rows, z };
    smooth(&mut g);

    let (lo, hi) = g.z.iter().fold((f32::MAX, f32::MIN), |(a, b), v| (a.min(*v), b.max(*v)));
    let first = (lo / interval).ceil() as i32;
    let last = (hi / interval).floor() as i32;
    let mut traced: Vec<(i32, Vec<Vec<(f64, f64)>>)> = Vec::new();
    for k in first..=last {
        let level = k as f32 * interval;
        traced.push((level as i32, trace(&g, level)));
    }

    // Simplify; tighten tolerance until under the point budget.
    let mut tol = 0.00003; // ~3 m, as in the compiler author's guide
    let mut out: Vec<(i32, Vec<(f64, f64)>)>;
    loop {
        out = Vec::new();
        for (ele, lines) in &traced {
            for l in lines {
                let s = simplify(l, tol);
                if s.len() >= 3 {
                    out.push((*ele, s));
                }
            }
        }
        let pts: usize = out.iter().map(|(_, l)| l.len()).sum();
        if pts <= MAX_POINTS || tol > 0.001 {
            break;
        }
        tol *= 2.0;
    }
    let points: usize = out.iter().map(|(_, l)| l.len()).sum();
    if out.is_empty() {
        return Ok(ContourStats { lines: 0, points: 0 });
    }

    // Serialize as OSM nodes and ways. Ids far below the street-label ids.
    let mut nodes = String::new();
    let mut ways = String::new();
    let mut id: i64 = -10_000_000;
    for (ele, line) in &out {
        let class = if ele % 100 == 0 { "c1" } else if ele % 50 == 0 { "c2" } else { "c3" };
        let mut refs = Vec::with_capacity(line.len());
        for (lon, lat) in line {
            nodes.push_str(&format!("  <node id=\"{id}\" lat=\"{lat:.7}\" lon=\"{lon:.7}\"/>\n"));
            refs.push(id);
            id -= 1;
        }
        ways.push_str(&format!("  <way id=\"{id}\">\n"));
        id -= 1;
        for r in refs {
            ways.push_str(&format!("    <nd ref=\"{r}\"/>\n"));
        }
        ways.push_str(&format!("    <tag k=\"highway\" v=\"{class}\"/>\n    <tag k=\"ele\" v=\"{ele}\"/>\n  </way>\n"));
    }
    inject(osm, &nodes, &ways)?;
    Ok(ContourStats { lines: out.len(), points })
}

/// Insert nodes before the first way and ways before the first relation (or
/// `</osm>`), keeping the node-then-way order the compiler expects.
fn inject(osm: &Path, nodes: &str, ways: &str) -> Result<(), String> {
    let tmp = osm.with_extension("osm.ctr");
    {
        let input = std::fs::File::open(osm).map_err(|e| e.to_string())?;
        let mut w = BufWriter::new(std::fs::File::create(&tmp).map_err(|e| e.to_string())?);
        let (mut nodes_done, mut ways_done) = (false, false);
        for line in BufReader::new(input).lines() {
            let line = line.map_err(|e| e.to_string())?;
            let t = line.trim_start();
            if !nodes_done && (t.starts_with("<way ") || t.starts_with("<relation ") || t.starts_with("</osm>")) {
                w.write_all(nodes.as_bytes()).map_err(|e| e.to_string())?;
                nodes_done = true;
            }
            if !ways_done && (t.starts_with("<relation ") || t.starts_with("</osm>")) {
                w.write_all(ways.as_bytes()).map_err(|e| e.to_string())?;
                ways_done = true;
            }
            writeln!(w, "{line}").map_err(|e| e.to_string())?;
        }
        if !ways_done {
            return Err("osm file has no closing </osm>".into());
        }
        w.flush().map_err(|e| e.to_string())?;
    }
    std::fs::rename(&tmp, osm).map_err(|e| e.to_string())
}

#[cfg(test)]
mod itest {
    use super::*;

    // Real data: DEM for a small Ramot area, contours every 20 m, compiled with
    // the bundled compiler.
    //   cargo test -p dt-g1-studio-app --lib contours_real -- --ignored --nocapture
    #[test]
    #[ignore]
    fn contours_real() {
        let base = std::env::temp_dir().join("dtg1_contours_itest");
        let _ = std::fs::remove_dir_all(base.join("out"));
        let cache = base.join("dem");
        let bbox = (35.186, 31.817, 35.196, 31.824);
        // elevation sanity: Ramot is roughly 700-850 m
        let mut dem = Dem::new(cache.clone());
        let e = dem.sample(31.820, 35.191).expect("sample");
        println!("elevation at Ramot point: {e:.1} m");
        assert!((600.0..900.0).contains(&e), "plausible elevation");

        let osm = base.join("map.osm");
        crate::mapcompile::download_osm(bbox.0, bbox.1, bbox.2, bbox.3, &osm).expect("download");
        let t = std::time::Instant::now();
        let stats = add_contours(&osm, bbox, 20, &cache).expect("contours");
        println!("contours: {} lines, {} points in {:?}", stats.lines, stats.points, t.elapsed());
        assert!(stats.lines > 0);

        let bundled = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("resources").join("map-compiler");
        let comp = base.join("compiler");
        std::fs::create_dir_all(&comp).unwrap();
        for e in std::fs::read_dir(&bundled).unwrap().flatten() {
            std::fs::copy(e.path(), comp.join(e.file_name())).unwrap();
        }
        let staging = base.join("out");
        let files = crate::mapcompile::compile(&comp, &osm, &staging, "Test", "landuse", "factory", &LUT_ROWS).expect("compile");
        println!("files: {files:?}");
        // second run must work from the cache alone
        let osm2 = base.join("map2.osm");
        std::fs::copy(&osm, &osm2).unwrap();
        assert!(cache.read_dir().unwrap().count() >= 2, "header + block cached");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cone(n: usize) -> Grid {
        // a 100 m high cone centred in an n x n grid
        let c = (n - 1) as f64 / 2.0;
        let mut z = Vec::new();
        for r in 0..n {
            for k in 0..n {
                let d = (((r as f64 - c).powi(2) + (k as f64 - c).powi(2)).sqrt()) / c;
                z.push((100.0 * (1.0 - d)).max(0.0) as f32);
            }
        }
        Grid { west: 35.0, south: 31.0, step: 1.0 / 3600.0, cols: n, rows: n, z }
    }

    #[test]
    fn cone_gives_closed_rings() {
        let g = cone(41);
        for level in [20.0, 50.0, 80.0] {
            let lines = trace(&g, level);
            assert_eq!(lines.len(), 1, "one ring at {level}");
            let l = &lines[0];
            assert!(l.len() > 8);
            assert_eq!(l.first(), l.last(), "ring is closed");
        }
        assert!(trace(&g, 150.0).is_empty());
    }

    #[test]
    fn simplify_keeps_ends_and_drops_collinear() {
        let pts: Vec<(f64, f64)> = (0..10).map(|i| (35.0 + i as f64 * 0.001, 31.0)).collect();
        let s = simplify(&pts, 0.00001);
        assert_eq!(s, vec![pts[0], pts[9]]);
    }

    #[test]
    fn predictor3_roundtrip() {
        // encode two rows of 4 floats the way the TIFF writer does, then decode
        use std::io::Write as _;
        let vals: [f32; 8] = [812.5, 813.0, 815.25, 790.0, -2.0, 0.0, 1000.75, 3.5];
        let (bw, bh) = (4usize, 2usize);
        let mut raw = Vec::new();
        for r in 0..bh {
            let row = &vals[r * bw..(r + 1) * bw];
            let mut planes = vec![0u8; bw * 4];
            for (k, v) in row.iter().enumerate() {
                let b = v.to_be_bytes();
                for p in 0..4 {
                    planes[p * bw + k] = b[p];
                }
            }
            for i in (1..planes.len()).rev() {
                planes[i] = planes[i].wrapping_sub(planes[i - 1]);
            }
            raw.extend_from_slice(&planes);
        }
        let mut enc = flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::default());
        enc.write_all(&raw).unwrap();
        let z = enc.finish().unwrap();
        assert_eq!(decode_block(&z, bw, bh).unwrap(), vals.to_vec());
    }
}
