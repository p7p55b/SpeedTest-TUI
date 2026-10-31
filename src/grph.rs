pub struct GgGrd {
    pub chs: Vec<Vec<char>>,
    pub clrs: Vec<Vec<u8>>,
    pub lbl_l: String,
    pub lbl_c: String,
    pub lbl_r: String,
    pub top_marks: Vec<(u16, &'static str, f64)>,
}

/// Bresenham continuous integer line algorithm on Braille subpixel grid
fn drw_ln<F>(mut x0: i32, mut y0: i32, x1: i32, y1: i32, lyr: u8, st_dt: &mut F)
where
    F: FnMut(i32, i32, u8),
{
    let dx = (x1 - x0).abs();
    let dy = -(y1 - y0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;

    loop {
        st_dt(x0, y0, lyr);
        if x0 == x1 && y0 == y1 {
            break;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x0 += sx;
        }
        if e2 <= dx {
            err += dx;
            y0 += sy;
        }
    }
}

/// Smooth progress bar with 1/8th sub-character block resolution
pub fn mk_br(w: usize, prg: f64) -> String {
    if w < 4 {
        return String::new();
    }
    let nw = w - 2;
    let clmp = prg.clamp(0.0, 1.0);
    let total_eighths = (clmp * (nw as f64 * 8.0)).round() as usize;
    let full_chars = total_eighths / 8;
    let remainder = total_eighths % 8;
    let empty_chars = nw.saturating_sub(full_chars + if remainder > 0 { 1 } else { 0 });

    let eighth_chars = [
        ' ', '\u{258f}', '\u{258e}', '\u{258d}', '\u{258c}', '\u{258b}', '\u{258a}', '\u{2589}',
    ];

    let mut rs = String::with_capacity(w * 4);
    rs.push('[');
    for _ in 0..full_chars {
        rs.push('\u{2588}');
    }
    if remainder > 0 {
        rs.push(eighth_chars[remainder]);
    }
    for _ in 0..empty_chars {
        rs.push('\u{2591}');
    }
    rs.push(']');
    rs
}

/// High-tech Braille curve with crisp crest line, shaded stipple area, and peak line
pub fn mk_brll(w: usize, h: usize, smpls: &[f64], pk: f64) -> Vec<String> {
    if w == 0 || h == 0 {
        return Vec::new();
    }
    let dw = w * 2;
    let dh = h * 4;
    let mut pts = vec![0.0; dw];

    if !smpls.is_empty() {
        let ln_s = smpls.len();
        if ln_s == 1 {
            pts.fill(smpls[0]);
        } else {
            for cl in 0..dw {
                let ps = (cl as f64 / (dw - 1) as f64) * (ln_s - 1) as f64;
                let p0 = ps as usize;
                let p1 = (p0 + 1).min(ln_s - 1);
                let frc = ps - p0 as f64;
                pts[cl] = smpls[p0] * (1.0 - frc) + smpls[p1] * frc;
            }
        }
    }

    let mut mx = if pk > 0.0 { pk } else { 1.0 };
    if pk <= 0.0 && !pts.is_empty() {
        let p_mx = pts.iter().copied().fold(0.0_f64, f64::max);
        if p_mx > 0.0 {
            mx = p_mx;
        }
    }
    if mx <= 0.0 {
        mx = 1.0;
    }

    let mut grd = vec![vec![0u8; w]; h];
    let l_bts = [0x01u8, 0x02, 0x04, 0x40];
    let r_bts = [0x08u8, 0x10, 0x20, 0x80];

    let mut st_dt = |px: i32, py: i32| {
        if px < 0 || px >= dw as i32 || py < 0 || py >= dh as i32 {
            return;
        }
        let cx = (px / 2) as usize;
        let r_sd = px % 2 == 1;
        let cy = (py / 4) as usize;
        let dy = (py % 4) as usize;
        grd[cy][cx] |= if r_sd { r_bts[dy] } else { l_bts[dy] };
    };

    // Calculate y coordinates for crest
    let mut y_pts = vec![0i32; dw];
    for c_ndx in 0..dw {
        let nrm = (pts[c_ndx] / mx).clamp(0.0, 1.0);
        let fll = (nrm * (dh - 1) as f64) as i32;
        let y_rv = (dh as i32 - 1) - fll;
        y_pts[c_ndx] = y_rv;
    }

    // 1. Shaded area below the curve with elegant stippling
    for c_ndx in 0..dw {
        let crest_y = y_pts[c_ndx];
        for py in crest_y..(dh as i32) {
            let dist_from_crest = py - crest_y;
            // First 2 dots below crest are solid, below that alternating stipple pattern
            if dist_from_crest <= 2 || (c_ndx + py as usize) % 2 == 0 {
                st_dt(c_ndx as i32, py);
            }
        }
    }

    // 2. Continuous crest line using Bresenham connecting adjacent points
    for c_ndx in 0..(dw.saturating_sub(1)) {
        let x0 = c_ndx as i32;
        let y0 = y_pts[c_ndx];
        let x1 = (c_ndx + 1) as i32;
        let y1 = y_pts[c_ndx + 1];
        drw_ln(x0, y0, x1, y1, 1, &mut |x, y, _| st_dt(x, y));
    }

    // 3. Peak reference dotted line across the graph
    if pk > 0.0 && mx > 0.0 {
        let pk_nrm = (pk / mx).clamp(0.0, 1.0);
        let pk_y = (dh as i32 - 1) - (pk_nrm * (dh - 1) as f64) as i32;
        for cl in (0..dw).step_by(4) {
            st_dt(cl as i32, pk_y);
        }
    }

    let mut rs = Vec::with_capacity(h);
    for r in 0..h {
        let mut ln = String::with_capacity(w * 3);
        for c in 0..w {
            let bt = grd[r][c];
            let ch = char::from_u32(0x2800 + bt as u32).unwrap_or(' ');
            ln.push(ch);
        }
        rs.push(ln);
    }
    rs
}

/// Realistic analog cockpit gauge with precision tapered needle, dual arcs,
/// logarithmic major/minor ticks, peak marker, and counterweight
pub fn mk_gg(w: usize, h: usize, mbps: f64, pk: f64) -> GgGrd {
    if w == 0 || h == 0 {
        return GgGrd {
            chs: Vec::new(),
            clrs: Vec::new(),
            lbl_l: "0".to_string(),
            lbl_c: "[ -- Mbps ]".to_string(),
            lbl_r: "10G".to_string(),
            top_marks: Vec::new(),
        };
    }

    let h_draw = if h >= 5 { h - 1 } else { h };
    let y_offset_dots = if h >= 5 { 4i32 } else { 0i32 };

    let dw = w * 2;
    let dh = h_draw * 4;
    let mut grd = vec![vec![0u8; w]; h];
    let mut tp_grd = vec![vec![0u8; w]; h];
    let l_bts = [0x01u8, 0x02, 0x04, 0x40];
    let r_bts = [0x08u8, 0x10, 0x20, 0x80];

    let mut st_dt = |px: i32, py: i32, lyr: u8| {
        let actual_py = py + y_offset_dots;
        if px < 0 || px >= dw as i32 || actual_py < 0 || actual_py >= (h * 4) as i32 {
            return;
        }
        let cx_n = (px / 2) as usize;
        let r_sd = px % 2 == 1;
        let cy_n = (actual_py / 4) as usize;
        let dy_n = (actual_py % 4) as usize;
        grd[cy_n][cx_n] |= if r_sd { r_bts[dy_n] } else { l_bts[dy_n] };
        if lyr > tp_grd[cy_n][cx_n] {
            tp_grd[cy_n][cx_n] = lyr;
        }
    };

    // Sweep: 2.20 rad (~126 deg each side, total 252 deg arc)
    let swp_mx = 2.20_f64;
    let cx = (dw as f64 - 1.0) * 0.5;
    let rx_mx = (dw as f64 - 3.0) * 0.5;
    let ry_mx = (dh as f64 - 3.0) / 1.54;
    let rd = 4.0_f64.max(rx_mx.min(ry_mx));
    let cy = 1.0 + rd;

    let vl = mbps.max(0.0);
    let mx_scl = 10000.0_f64.max(pk.max(vl));
    let mut nrm = (1.0 + vl).log10() / (1.0 + mx_scl).log10();
    nrm = nrm.clamp(0.0, 1.0);
    let ndl_ng = -swp_mx + nrm * (2.0 * swp_mx);

    // 1. Dual Outer & Inner Scale Arcs
    let stp_ng = 0.012_f64.max(0.35 / rd);
    let mut ng = -swp_mx;
    while ng <= swp_mx + 0.001 {
        let is_active = vl > 0.05 && ng <= ndl_ng;
        let lyr = if is_active { 1 } else { 0 };
        let sn = ng.sin();
        let cs = ng.cos();

        // Outer arc
        st_dt((cx + rd * sn).round() as i32, (cy - rd * cs).round() as i32, lyr);

        // Inner contour for solid bezel feel if radius allows
        if rd >= 7.0 {
            st_dt(
                (cx + (rd - 0.9) * sn).round() as i32,
                (cy - (rd - 0.9) * cs).round() as i32,
                lyr,
            );
        }
        ng += stp_ng;
    }

    // 2. Realistic Logarithmic Ticks (Major & Minor)
    // Key values: 0, 10M, 50M, 100M, 250M, 500M, 1G, 2.5G, 5G, 10G
    let tick_vals: &[(f64, bool)] = &[
        (0.0, true),
        (5.0, false),
        (10.0, true),
        (25.0, false),
        (50.0, false),
        (100.0, true),
        (250.0, false),
        (500.0, true),
        (1000.0, true),
        (2500.0, true),
        (5000.0, true),
        (10000.0, true),
    ];

    for &(t_val, is_major) in tick_vals {
        let t_nrm = (1.0 + t_val).log10() / (1.0 + 10000.0_f64).log10();
        let t_ng = -swp_mx + t_nrm * (2.0 * swp_mx);
        let sn = t_ng.sin();
        let cs = t_ng.cos();
        let is_act = vl > 0.05 && t_ng <= ndl_ng;
        let lyr = if is_act { 1 } else { 0 };

        let t_len = if is_major {
            3.5_f64.min(rd * 0.34)
        } else {
            1.8_f64.min(rd * 0.18)
        };

        let mut d = 0.6_f64;
        while d <= t_len {
            st_dt(
                (cx + (rd - d) * sn).round() as i32,
                (cy - (rd - d) * cs).round() as i32,
                lyr,
            );
            d += 0.35;
        }

        // Distinctive double-tick for 2.5G
        if t_val == 2500.0 && rd >= 7.0 {
            let p_sn = cs;
            let p_cs = -sn;
            let mut d = 0.6_f64;
            while d <= t_len {
                st_dt(
                    (cx + (rd - d) * sn + 0.7 * p_sn).round() as i32,
                    (cy - (rd - d) * cs + 0.7 * p_cs).round() as i32,
                    lyr,
                );
                d += 0.35;
            }
        }
    }

    // 3. Peak Hold Marker on Arc
    if pk > 0.0 {
        let pk_nrm = ((1.0 + pk).log10() / (1.0 + mx_scl).log10()).clamp(0.0, 1.0);
        let pk_ng = -swp_mx + pk_nrm * (2.0 * swp_mx);
        let pk_sn = pk_ng.sin();
        let pk_cs = pk_ng.cos();
        for d in [0.4, 0.9, 1.4, 2.0] {
            st_dt(
                (cx + (rd - d) * pk_sn).round() as i32,
                (cy - (rd - d) * pk_cs).round() as i32,
                3, // Layer 3: Peak Warning
            );
        }
    }

    // 4. Center Hub & Brass Pivot Mechanism
    let h_cx = cx.round() as i32;
    let h_cy = cy.round() as i32;
    let hub_r = 2.2_f64.min(rd * 0.22);
    let hub_r_i = hub_r.round() as i32;

    for dy in -hub_r_i..=hub_r_i {
        for dx in -hub_r_i..=hub_r_i {
            let dist_sq = dx * dx + dy * dy;
            if dist_sq <= (hub_r * hub_r) as i32 {
                let lyr = if dist_sq <= 1 { 4 } else { 2 };
                st_dt(h_cx + dx, h_cy + dy, lyr);
            }
        }
    }

    // 5. Tapered Needle with Tail Counterweight
    let n_sn = ndl_ng.sin();
    let n_cs = ndl_ng.cos();
    let p_sn = n_cs;  // Perpendicular unit vector x
    let p_cs = -n_sn; // Perpendicular unit vector y

    let ndl_len = 3.0_f64.max(rd - 1.2);
    let tail_len = 2.2_f64.min(rd * 0.26);

    // Tip coordinate
    let tip_x = (cx + ndl_len * n_sn).round() as i32;
    let tip_y = (cy - ndl_len * n_cs).round() as i32;

    // Base left & right coordinates (at hub edge)
    let base_d = hub_r * 0.8;
    let base_w = 0.95_f64;
    let base_lx = (cx + base_d * n_sn - base_w * p_sn).round() as i32;
    let base_ly = (cy - base_d * n_cs - base_w * p_cs).round() as i32;
    let base_rx = (cx + base_d * n_sn + base_w * p_sn).round() as i32;
    let base_ry = (cy - base_d * n_cs + base_w * p_cs).round() as i32;

    // Draw solid tapered needle using Bresenham lines
    drw_ln(h_cx, h_cy, tip_x, tip_y, 2, &mut st_dt);
    drw_ln(base_lx, base_ly, tip_x, tip_y, 2, &mut st_dt);
    drw_ln(base_rx, base_ry, tip_x, tip_y, 2, &mut st_dt);

    // Mid-needle filler for solid body if needle is large
    if rd >= 8.0 {
        let mid_d = ndl_len * 0.45;
        let mid_w = base_w * 0.55;
        let mid_lx = (cx + mid_d * n_sn - mid_w * p_sn).round() as i32;
        let mid_ly = (cy - mid_d * n_cs - mid_w * p_cs).round() as i32;
        let mid_rx = (cx + mid_d * n_sn + mid_w * p_sn).round() as i32;
        let mid_ry = (cy - mid_d * n_cs + mid_w * p_cs).round() as i32;
        drw_ln(base_lx, base_ly, mid_rx, mid_ry, 2, &mut st_dt);
        drw_ln(base_rx, base_ry, mid_lx, mid_ly, 2, &mut st_dt);
    }

    // Needle Counterweight Tail (diamond shape at back of pivot)
    let tail_tip_x = (cx - tail_len * n_sn).round() as i32;
    let tail_tip_y = (cy + tail_len * n_cs).round() as i32;
    let tail_mid_d = tail_len * 0.6;
    let tail_w = 1.1_f64;
    let tail_lx = (cx - tail_mid_d * n_sn - tail_w * p_sn).round() as i32;
    let tail_ly = (cy + tail_mid_d * n_cs - tail_w * p_cs).round() as i32;
    let tail_rx = (cx - tail_mid_d * n_sn + tail_w * p_sn).round() as i32;
    let tail_ry = (cy + tail_mid_d * n_cs + tail_w * p_cs).round() as i32;

    drw_ln(h_cx, h_cy, tail_tip_x, tail_tip_y, 5, &mut st_dt);
    drw_ln(tail_lx, tail_ly, tail_tip_x, tail_tip_y, 5, &mut st_dt);
    drw_ln(tail_rx, tail_ry, tail_tip_x, tail_tip_y, 5, &mut st_dt);
    drw_ln(h_cx, h_cy, tail_lx, tail_ly, 5, &mut st_dt);
    drw_ln(h_cx, h_cy, tail_rx, tail_ry, 5, &mut st_dt);

    // Convert Braille dot matrix to UTF-8 characters
    let mut chs = vec![vec![' '; w]; h];
    for r in 0..h {
        for c in 0..w {
            let bt = grd[r][c];
            chs[r][c] = char::from_u32(0x2800 + bt as u32).unwrap_or(' ');
        }
    }

    let lbl_c = if mbps < 0.0 {
        "[ -- Mbps ]".to_string()
    } else if mbps >= 1000.0 {
        format!("[ {:.2} Gbps ]", mbps / 1000.0)
    } else {
        format!("[ {:.1} Mbps ]", mbps)
    };

    let mut top_marks = Vec::new();
    if h >= 5 {
        if w >= 22 {
            let c1 = if w >= 25 { 1 } else { 0 };
            let c2 = (w as f64 * 0.33) as u16;
            let c3 = (w as f64 * 0.63) as u16;
            let c4 = (w.saturating_sub(4)) as u16;
            top_marks.push((c1, "10M", 10.0));
            top_marks.push((c2, "100M", 100.0));
            top_marks.push((c3, "1G", 1000.0));
            top_marks.push((c4, "2.5G", 2500.0));
        } else if w >= 16 {
            let c1 = 0;
            let c2 = ((w.saturating_sub(4)) / 2) as u16;
            let c3 = (w.saturating_sub(4)) as u16;
            top_marks.push((c1, "10M", 10.0));
            top_marks.push((c2, "100M", 100.0));
            top_marks.push((c3, "2.5G", 2500.0));
        } else if w >= 12 {
            let c1 = 0;
            let c2 = (w.saturating_sub(4)) as u16;
            top_marks.push((c1, "100M", 100.0));
            top_marks.push((c2, "2.5G", 2500.0));
        }
    }

    GgGrd {
        chs,
        clrs: tp_grd,
        lbl_l: "0".to_string(),
        lbl_c,
        lbl_r: "10G".to_string(),
        top_marks,
    }
}

pub fn fmt_spd(bw: f64) -> String {
    if bw < 0.0 {
        return "-- Mbps".to_string();
    }
    let mbps = (bw * 8.0) / 1_000_000.0;
    if mbps >= 1000.0 {
        format!("{:.2} Gbps", mbps / 1000.0)
    } else {
        format!("{:.1} Mbps", mbps)
    }
}

pub fn fmt_sz(byt: i64) -> String {
    if byt < 0 {
        return "-- MB".to_string();
    }
    let mb = byt as f64 / 1_000_000.0;
    if mb >= 1000.0 {
        format!("{:.2} GB", mb / 1000.0)
    } else {
        format!("{:.1} MB", mb)
    }
}

pub fn fmt_ms(ms: f64) -> String {
    if ms < 0.0 {
        "-- ms".to_string()
    } else {
        format!("{:.1} ms", ms)
    }
}
