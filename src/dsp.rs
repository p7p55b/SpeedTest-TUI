use crate::engn::{Ngn, TstSt};
use crate::grph::{fmt_ms, fmt_spd, fmt_sz, mk_br, mk_brll, mk_gg};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{Block, Clear, Paragraph},
    Frame,
};

pub const CLR_BG: Color = Color::Rgb(84, 94, 96); // #545e60 (slate khaki)
pub const CLR_PANE_BG: Color = Color::Rgb(42, 53, 54); // #2a3536 (slate grey)
pub const CLR_MID: Color = Color::Rgb(22, 35, 38); // #162326 (deep obsidian petrol)
pub const CLR_FG: Color = Color::Rgb(235, 230, 202); // #ebe6ca (bone parchment)
pub const CLR_LN: Color = Color::Rgb(205, 204, 178); // #cdccb2 (wireframe sand bone)
pub const CLR_ACC: Color = Color::Rgb(151, 177, 170); // #97b1aa (faded sage teal)
pub const CLR_GRN: Color = Color::Rgb(74, 222, 128); // #4ade80 (phosphor green)
pub const CLR_DNG: Color = Color::Rgb(239, 68, 68); // #ef4444 (signal red)
pub const CLR_WRN: Color = Color::Rgb(254, 240, 138); // #fef08a (warning yellow)
pub const CLR_OFF: Color = Color::Rgb(102, 102, 102); // #666666 (muted grey)

#[derive(Clone, Debug)]
pub struct CnftPt {
    pub x: f64,
    pub y: f64,
    pub vx: f64,
    pub vy: f64,
    pub clr: Color,
    pub ch_ndx: usize,
    pub lf: i32,
}

pub struct Dsp {
    pub shw_mdl: bool,
    pub sl_ndx: usize,
    pub rx_ndl: f64,
    pub rx_vel: f64,
    pub tx_ndl: f64,
    pub tx_vel: f64,
    pub cnft_pts: Vec<CnftPt>,
    pub cnft_tmr: i32,
    pub lst_rcrd_cnt: i32,
}

impl Default for Dsp {
    fn default() -> Self {
        Self::new()
    }
}

impl Dsp {
    pub fn new() -> Self {
        Self {
            shw_mdl: false,
            sl_ndx: 0,
            rx_ndl: 0.0,
            rx_vel: 0.0,
            tx_ndl: 0.0,
            tx_vel: 0.0,
            cnft_pts: Vec::new(),
            cnft_tmr: 0,
            lst_rcrd_cnt: 0,
        }
    }

    pub fn upd_ndls(&mut self, st: &TstSt) {
        let rx_tgt = if st.rx_bw >= 0.0 {
            st.rx_bw * 8.0 / 1_000_000.0
        } else {
            0.0
        };
        let tx_tgt = if st.tx_bw >= 0.0 {
            st.tx_bw * 8.0 / 1_000_000.0
        } else {
            0.0
        };

        // Realistic spring-damper inertia with subtle mechanical overshoot
        let diff_rx = rx_tgt - self.rx_ndl;
        self.rx_vel = (self.rx_vel + diff_rx * 0.28) * 0.68;
        self.rx_ndl += self.rx_vel;
        if self.rx_ndl < 0.0 {
            self.rx_ndl = 0.0;
            self.rx_vel = 0.0;
        }

        let diff_tx = tx_tgt - self.tx_ndl;
        self.tx_vel = (self.tx_vel + diff_tx * 0.28) * 0.68;
        self.tx_ndl += self.tx_vel;
        if self.tx_ndl < 0.0 {
            self.tx_ndl = 0.0;
            self.tx_vel = 0.0;
        }

        if st.rcrd_cnt != self.lst_rcrd_cnt {
            self.lst_rcrd_cnt = st.rcrd_cnt;
            self.strt_cnft(24, 80);
        }
    }

    pub fn strt_cnft(&mut self, my: u16, mx: u16) {
        self.cnft_pts.clear();
        self.cnft_tmr = 85;
        let clrs = [CLR_GRN, CLR_WRN, CLR_ACC, CLR_DNG, CLR_GRN, CLR_ACC];
        let srcs = [
            mx as f64 * 0.22,
            mx as f64 * 0.50,
            mx as f64 * 0.78,
        ];

        for n in 0..96 {
            let src_x = srcs[n % 3] + ((n * 13) % 9) as f64 - 4.0;
            let src_y = (my / 2) as f64 + ((n * 7) % 5) as f64 - 2.0;
            let ng = -std::f64::consts::PI * 0.15 - (((n * 19) % 100) as f64 / 100.0) * (std::f64::consts::PI * 0.70);
            let spd = 0.6 + (((n * 23) % 100) as f64 / 100.0) * 1.9;
            self.cnft_pts.push(CnftPt {
                x: src_x,
                y: src_y,
                vx: ng.cos() * spd * 1.6,
                vy: ng.sin() * spd,
                clr: clrs[n % 6],
                ch_ndx: n % 7,
                lf: 45 + ((n * 17) % 45) as i32,
            });
        }
    }

    pub fn rndr(&mut self, f: &mut Frame, ngn: &Ngn) {
        let sz = f.area();
        let my = sz.height;
        let mx = sz.width;

        if my < 20 || mx < 74 {
            let wrn_st = Style::default()
                .fg(CLR_WRN)
                .bg(CLR_MID)
                .add_modifier(Modifier::BOLD);
            f.render_widget(
                Paragraph::new("[ TERMINAL WINDOW TOO SMALL (MIN 74x20) ]").style(wrn_st),
                sz,
            );
            return;
        }

        let st = ngn.gt_st();
        self.upd_ndls(&st);

        // Fill background
        let bg_st = Style::default().bg(CLR_BG);
        f.render_widget(Block::default().style(bg_st), sz);

        // Header
        self.drw_hdr(f, mx, &st);

        let tp_h = 6u16;
        let tp_w1 = mx / 2;
        let tp_w2 = mx - tp_w1;

        // Top panels: Latency & Target Server
        self.drw_ltnc(f, Rect::new(0, 1, tp_w1, tp_h), &st);
        self.drw_srv(f, Rect::new(tp_w1, 1, tp_w2, tp_h), &st, ngn);

        // Middle panels: Download & Upload
        let md_y = 1 + tp_h;
        let rm_h = my.saturating_sub(md_y);
        let bt_h = if my < 26 { 8 } else { 10 };
        let md_h = (rm_h.saturating_sub(bt_h)).max(9);

        let md_w1 = mx / 2;
        let md_w2 = mx - md_w1;
        self.drw_rx(f, Rect::new(0, md_y, md_w1, md_h), &st);
        self.drw_tx(f, Rect::new(md_w1, md_y, md_w2, md_h), &st);

        // Bottom panels: History & Controls (and dedicated full-width Result URL bar when present)
        let bt_y = md_y + md_h;
        let ct_bt_h = my.saturating_sub(bt_y);
        if ct_bt_h >= 5 {
            if !st.rs_lnk.is_empty() && ct_bt_h >= 7 {
                self.drw_rs_lnk(f, Rect::new(0, bt_y, mx, 3), &st);
                let sub_y = bt_y + 3;
                let sub_h = ct_bt_h - 3;
                let bt_w1 = (mx as f64 * 0.55) as u16;
                let bt_w2 = mx.saturating_sub(bt_w1).max(2);
                self.drw_hst(f, Rect::new(0, sub_y, bt_w1, sub_h), ngn);
                self.drw_ctrl(f, Rect::new(bt_w1, sub_y, bt_w2, sub_h), &st);
            } else {
                let needed_ctrl_w = if !st.rs_lnk.is_empty() {
                    (st.rs_lnk.len() as u16 + 8).max(30)
                } else {
                    30
                };
                let min_hst_w = if mx > 110 { 50 } else { 38 };
                let bt_w2 = needed_ctrl_w.min(mx.saturating_sub(min_hst_w)).max(28);
                let bt_w1 = mx.saturating_sub(bt_w2);
                self.drw_hst(f, Rect::new(0, bt_y, bt_w1, ct_bt_h), ngn);
                self.drw_ctrl(f, Rect::new(bt_w1, bt_y, bt_w2, ct_bt_h), &st);
            }
        }

        // Modal
        if self.shw_mdl {
            self.drw_mdl(f, my, mx, ngn);
        }

        // Confetti
        if self.cnft_tmr > 0 {
            self.drw_cnft(f, my, mx, &st);
        }
    }

    fn drw_hdr(&self, f: &mut Frame, w: u16, st: &TstSt) {
        let ctv = matches!(st.st.as_str(), "PING" | "DL" | "UL" | "DONE");
        let dt_clr = if ctv { CLR_GRN } else { CLR_OFF };

        let buf = f.buffer_mut();
        let hdr_st = Style::default().fg(CLR_FG).bg(CLR_MID);
        for x in 0..w {
            buf.set_string(x, 0, " ", hdr_st);
        }

        buf.set_string(
            1,
            0,
            "\u{25cf}",
            Style::default().fg(dt_clr).bg(CLR_MID).add_modifier(Modifier::BOLD),
        );

        let ttl = " SPEEDTEST TUI // OOKLA TERMINAL UI ";
        buf.set_string(
            2,
            0,
            ttl,
            Style::default().fg(CLR_FG).bg(CLR_MID).add_modifier(Modifier::BOLD),
        );

        let prv = if st.prvdr.is_empty() { "detecting" } else { &st.prvdr };
        let rmt = if st.rmt_ddr.is_empty() { "--" } else { &st.rmt_ddr };
        let ifc = if st.nc_nm.is_empty() { "--" } else { &st.nc_nm };
        let dtls = format!("isp: {} | target_ip: {} | iface: {} ", prv, rmt, ifc);

        let dx = w.saturating_sub(dtls.len() as u16);
        if dx > (ttl.len() as u16 + 3) {
            buf.set_string(
                dx,
                0,
                &dtls,
                Style::default().fg(CLR_ACC).bg(CLR_MID),
            );
        }
    }

    fn drw_bx(&self, buf: &mut Buffer, area: Rect, ttl: &str) {
        if area.width < 2 || area.height < 2 {
            return;
        }

        let pn_st = Style::default().fg(CLR_FG).bg(CLR_PANE_BG);
        let ln_st = Style::default().fg(CLR_LN).bg(CLR_PANE_BG);
        let ttl_st = Style::default()
            .fg(CLR_ACC)
            .bg(CLR_PANE_BG)
            .add_modifier(Modifier::BOLD);

        // Fill background
        for y in (area.y + 1)..(area.y + area.height - 1) {
            for x in (area.x + 1)..(area.x + area.width - 1) {
                buf.set_string(x, y, " ", pn_st);
            }
        }

        // Draw borders
        let w = area.width as usize;
        let mut hr_br = String::with_capacity(w * 3);
        for _ in 0..w.saturating_sub(2) {
            hr_br.push('\u{2500}');
        }

        let tp_ln = format!("\u{250c}{}\u{2510}", hr_br);
        let bt_ln = format!("\u{2514}{}\u{2518}", hr_br);

        buf.set_string(area.x, area.y, &tp_ln, ln_st);
        for y in (area.y + 1)..(area.y + area.height - 1) {
            buf.set_string(area.x, y, "\u{2502}", ln_st);
            buf.set_string(area.x + area.width - 1, y, "\u{2502}", ln_st);
        }
        buf.set_string(area.x, area.y + area.height - 1, &bt_ln, ln_st);

        // Draw title
        if !ttl.is_empty() {
            let t_str = format!(" [ {} ] ", ttl);
            if t_str.len() < w.saturating_sub(4) {
                buf.set_string(area.x + 2, area.y, &t_str, ttl_st);
            }
        }
    }

    fn drw_ltnc(&self, f: &mut Frame, area: Rect, st: &TstSt) {
        let buf = f.buffer_mut();
        self.drw_bx(buf, area, "LATENCY / JITTER");

        let pn_st = Style::default().fg(CLR_FG).bg(CLR_PANE_BG);
        let acc_st = Style::default().fg(CLR_ACC).bg(CLR_PANE_BG);

        let bf1 = format!(
            "ping: {:<12} jitter: {:<12}",
            fmt_ms(st.ltnc),
            fmt_ms(st.jttr)
        );
        let lss = if st.pkt_lss >= 0.0 { st.pkt_lss } else { 0.0 };
        let bf2 = format!("loss: {:.1} %", lss);

        let max_w = area.width.saturating_sub(4) as usize;
        buf.set_string(area.x + 2, area.y + 1, Self::clp(&bf1, max_w), pn_st);
        buf.set_string(area.x + 2, area.y + 2, Self::clp(&bf2, max_w), pn_st);

        let gh = area.height.saturating_sub(4) as usize;
        let gw = area.width.saturating_sub(4) as usize;
        if gh >= 1 && gw >= 4 {
            let lns = mk_brll(gw, gh, &st.ltnc_pts, 0.0);
            for (ndx, ln) in lns.iter().enumerate() {
                let py = area.y + 3 + ndx as u16;
                if py < area.y + area.height - 1 {
                    buf.set_string(area.x + 2, py, ln, acc_st);
                }
            }
        }
    }

    fn drw_srv(&self, f: &mut Frame, area: Rect, st: &TstSt, ngn: &Ngn) {
        let buf = f.buffer_mut();
        self.drw_bx(buf, area, "TARGET SERVER");

        let pn_st = Style::default().fg(CLR_FG).bg(CLR_PANE_BG);
        let btn_st = Style::default()
            .fg(CLR_ACC)
            .bg(CLR_PANE_BG)
            .add_modifier(Modifier::BOLD);

        let hs_sl = ngn.hs_sl_srv.lock().map(|h| *h).unwrap_or(false);
        let sl_srv = ngn.sl_srv.lock().ok().and_then(|s| s.clone());

        let srv_nm = if !st.srv_nm.is_empty() {
            st.srv_nm.as_str()
        } else if hs_sl {
            sl_srv.as_ref().map(|s| s.name.as_str()).unwrap_or("auto_selection")
        } else {
            "auto_selection"
        };

        let srv_lc = if !st.srv_lc.is_empty() {
            st.srv_lc.as_str()
        } else if hs_sl {
            sl_srv.as_ref().map(|s| s.location.as_str()).unwrap_or("closest_host")
        } else {
            "closest_host"
        };

        let srv_hst = if !st.srv_hst.is_empty() {
            st.srv_hst.as_str()
        } else if hs_sl {
            sl_srv.as_ref().map(|s| s.host.as_str()).unwrap_or("--")
        } else {
            "--"
        };

        let srv_prt = if st.srv_prt > 0 {
            st.srv_prt
        } else if hs_sl {
            sl_srv.as_ref().map(|s| s.port).unwrap_or(8080)
        } else {
            8080
        };

        let srv_k_str = if st.srv_k > 0 {
            st.srv_k.to_string()
        } else if hs_sl {
            sl_srv.as_ref().map(|s| s.id.to_string()).unwrap_or_else(|| "auto".to_string())
        } else {
            "auto".to_string()
        };

        let max_w = area.width.saturating_sub(4) as usize;
        let l1 = format!("name: {}", srv_nm);
        let l2 = format!("host: {} (port: {})", srv_hst, srv_prt);
        let l3 = format!("loc : {} | server_id: {}", srv_lc, srv_k_str);

        buf.set_string(area.x + 2, area.y + 1, Self::clp(&l1, max_w), pn_st);
        buf.set_string(area.x + 2, area.y + 2, Self::clp(&l2, max_w), pn_st);
        buf.set_string(area.x + 2, area.y + 3, Self::clp(&l3, max_w), pn_st);
        buf.set_string(area.x + 2, area.y + 4, Self::clp("[ S ] SELECT SERVER", max_w), btn_st);
    }

    fn drw_gg_widget(
        buf: &mut Buffer,
        x: u16,
        y: u16,
        h: usize,
        w: usize,
        mbps: f64,
        pk: f64,
        cp_ctv: Color,
        cp_ndl: Color,
    ) {
        if h < 2 || w < 6 {
            return;
        }
        let gg = mk_gg(w, h, mbps, pk);
        let ln_st = Style::default().fg(CLR_LN).bg(CLR_PANE_BG);
        let ctv_st = Style::default().fg(cp_ctv).bg(CLR_PANE_BG).add_modifier(Modifier::BOLD);
        let ndl_st = Style::default().fg(cp_ndl).bg(CLR_PANE_BG).add_modifier(Modifier::BOLD);
        let pk_st = Style::default().fg(CLR_DNG).bg(CLR_PANE_BG).add_modifier(Modifier::BOLD);
        let hub_st = Style::default().fg(CLR_FG).bg(CLR_PANE_BG).add_modifier(Modifier::BOLD);

        for r in 0..gg.chs.len() {
            for c in 0..gg.chs[r].len() {
                let lyr = gg.clrs[r][c];
                let st = match lyr {
                    2 => ndl_st,
                    1 => ctv_st,
                    3 => pk_st,
                    4 => hub_st,
                    5 => ln_st,
                    _ => ln_st,
                };
                let ch_str = gg.chs[r][c].to_string();
                buf.set_string(x + c as u16, y + r as u16, &ch_str, st);
            }
        }

        let bt_r = y + h as u16 - 1;

        // Render top graduation scale numbers (10M, 100M, 1G)
        for &(col, txt, thresh) in &gg.top_marks {
            let is_act = mbps >= thresh || pk >= thresh;
            let st = if is_act { ctv_st } else { ln_st };
            buf.set_string(x + col, y, txt, st);
        }

        // Render bottom graduation scale numbers (0, [ speed ], 10G)
        let is_0_act = mbps >= 0.05 || pk >= 0.05;
        let st_0 = if is_0_act { ctv_st } else { ln_st };
        buf.set_string(x + 1, bt_r, &gg.lbl_l, st_0);

        let rx_p = x + w as u16 - 1 - gg.lbl_r.len() as u16;
        if rx_p > x + 2 {
            let is_10g_act = mbps >= 9500.0 || pk >= 9500.0;
            let st_10g = if is_10g_act { ctv_st } else { ln_st };
            buf.set_string(rx_p, bt_r, &gg.lbl_r, st_10g);
        }

        let cx_p = x + (w.saturating_sub(gg.lbl_c.len()) / 2) as u16;
        if cx_p > x + 2 && (cx_p + gg.lbl_c.len() as u16) < rx_p {
            let badge_st = Style::default()
                .fg(cp_ctv)
                .bg(CLR_MID)
                .add_modifier(Modifier::BOLD);
            buf.set_string(cx_p, bt_r, &gg.lbl_c, badge_st);
        }
    }

    fn drw_rx(&self, f: &mut Frame, area: Rect, st: &TstSt) {
        let buf = f.buffer_mut();
        self.drw_bx(buf, area, "DOWNLOAD");

        let grn_bold = Style::default()
            .fg(CLR_GRN)
            .bg(CLR_PANE_BG)
            .add_modifier(Modifier::BOLD);
        let pn_st = Style::default().fg(CLR_FG).bg(CLR_PANE_BG);
        let acc_st = Style::default().fg(CLR_ACC).bg(CLR_PANE_BG);

        let max_w = area.width.saturating_sub(4) as usize;
        let l1 = format!("sync_rate: {}", fmt_spd(st.rx_bw));
        buf.set_string(area.x + 2, area.y + 1, Self::clp(&l1, max_w), grn_bold);

        let l2 = format!(
            "peak: {:.1} Mbps | avg: {:.1} Mbps | data: {}",
            st.rx_pk,
            st.rx_vg,
            fmt_sz(st.rx_byt)
        );
        buf.set_string(area.x + 2, area.y + 2, Self::clp(&l2, max_w), pn_st);

        let bw_br = mk_br(max_w, st.rx_prg);
        buf.set_string(area.x + 2, area.y + 3, &bw_br, acc_st);

        let gh = area.height.saturating_sub(5) as usize;
        let gw = max_w;
        let dsp_mbps = if st.rx_bw < 0.0 && self.rx_ndl < 0.05 {
            -1.0
        } else {
            self.rx_ndl
        };

        if gh >= 2 && gw >= 8 {
            if gw >= 26 {
                let gg_w = 26.min(14.max((gw * 46) / 100));
                let crv_x = area.x + 2 + gg_w as u16 + 1;
                let crv_w = gw.saturating_sub(gg_w + 1);

                Self::drw_gg_widget(
                    buf,
                    area.x + 2,
                    area.y + 4,
                    gh,
                    gg_w,
                    dsp_mbps,
                    st.rx_pk,
                    CLR_GRN,
                    CLR_WRN,
                );

                if crv_w >= 4 {
                    let lns = mk_brll(crv_w, gh, &st.rx_pts, st.rx_pk);
                    let grn_st = Style::default().fg(CLR_GRN).bg(CLR_PANE_BG);
                    for (ndx, ln) in lns.iter().enumerate() {
                        let py = area.y + 4 + ndx as u16;
                        if py < area.y + area.height - 1 {
                            buf.set_string(crv_x, py, ln, grn_st);
                        }
                    }
                }
            } else {
                Self::drw_gg_widget(
                    buf,
                    area.x + 2,
                    area.y + 4,
                    gh,
                    gw,
                    dsp_mbps,
                    st.rx_pk,
                    CLR_GRN,
                    CLR_WRN,
                );
            }
        }
    }

    fn drw_tx(&self, f: &mut Frame, area: Rect, st: &TstSt) {
        let buf = f.buffer_mut();
        self.drw_bx(buf, area, "UPLOAD");

        let acc_bold = Style::default()
            .fg(CLR_ACC)
            .bg(CLR_PANE_BG)
            .add_modifier(Modifier::BOLD);
        let pn_st = Style::default().fg(CLR_FG).bg(CLR_PANE_BG);
        let acc_st = Style::default().fg(CLR_ACC).bg(CLR_PANE_BG);

        let max_w = area.width.saturating_sub(4) as usize;
        let l1 = format!("sync_rate: {}", fmt_spd(st.tx_bw));
        buf.set_string(area.x + 2, area.y + 1, Self::clp(&l1, max_w), acc_bold);

        let l2 = format!(
            "peak: {:.1} Mbps | avg: {:.1} Mbps | data: {}",
            st.tx_pk,
            st.tx_vg,
            fmt_sz(st.tx_byt)
        );
        buf.set_string(area.x + 2, area.y + 2, Self::clp(&l2, max_w), pn_st);

        let bw_br = mk_br(max_w, st.tx_prg);
        buf.set_string(area.x + 2, area.y + 3, &bw_br, acc_st);

        let gh = area.height.saturating_sub(5) as usize;
        let gw = max_w;
        let dsp_mbps = if st.tx_bw < 0.0 && self.tx_ndl < 0.05 {
            -1.0
        } else {
            self.tx_ndl
        };

        if gh >= 2 && gw >= 8 {
            if gw >= 26 {
                let gg_w = 26.min(14.max((gw * 46) / 100));
                let crv_x = area.x + 2 + gg_w as u16 + 1;
                let crv_w = gw.saturating_sub(gg_w + 1);

                Self::drw_gg_widget(
                    buf,
                    area.x + 2,
                    area.y + 4,
                    gh,
                    gg_w,
                    dsp_mbps,
                    st.tx_pk,
                    CLR_ACC,
                    CLR_WRN,
                );

                if crv_w >= 4 {
                    let lns = mk_brll(crv_w, gh, &st.tx_pts, st.tx_pk);
                    let acc_crv_st = Style::default().fg(CLR_ACC).bg(CLR_PANE_BG);
                    for (ndx, ln) in lns.iter().enumerate() {
                        let py = area.y + 4 + ndx as u16;
                        if py < area.y + area.height - 1 {
                            buf.set_string(crv_x, py, ln, acc_crv_st);
                        }
                    }
                }
            } else {
                Self::drw_gg_widget(
                    buf,
                    area.x + 2,
                    area.y + 4,
                    gh,
                    gw,
                    dsp_mbps,
                    st.tx_pk,
                    CLR_ACC,
                    CLR_WRN,
                );
            }
        }
    }

    fn drw_hst(&self, f: &mut Frame, area: Rect, ngn: &Ngn) {
        let buf = f.buffer_mut();
        self.drw_bx(buf, area, "TEST HISTORY");

        let acc_bold = Style::default()
            .fg(CLR_ACC)
            .bg(CLR_PANE_BG)
            .add_modifier(Modifier::BOLD);
        let pn_st = Style::default().fg(CLR_FG).bg(CLR_PANE_BG);

        let w = area.width as usize;
        let hdr = if w > 65 {
            format!(
                "{:<19} {:<20} {:<9} {:<11} {:<10}",
                "TIME", "SERVER", "PING", "DL (Mbps)", "UL (Mbps)"
            )
        } else {
            format!(
                "{:<9} {:<16} {:<8} {:<8} {:<8}",
                "TIME", "SERVER", "PING", "DL", "UL"
            )
        };

        let max_w = area.width.saturating_sub(4) as usize;
        buf.set_string(area.x + 2, area.y + 1, Self::clp(&hdr, max_w), acc_bold);

        let lst = ngn.ld_hst();
        let mx_rws = area.height.saturating_sub(3) as usize;
        for (ndx, rc) in lst.iter().take(mx_rws).enumerate() {
            let tm = if w > 65 {
                Self::clp(&rc.tm, 19)
            } else if rc.tm.len() >= 19 {
                &rc.tm[11..19]
            } else {
                Self::clp(&rc.tm, 8)
            };
            let sv = Self::clp(&rc.srv, if w > 65 { 19 } else { 15 });
            let pn_bf = format!("{:.1} ms", rc.ltnc);
            let rx_bf = format!("{:.1}", rc.dl);
            let tx_bf = format!("{:.1}", rc.ul);

            let rw = if w > 65 {
                format!(
                    "{:<19} {:<20} {:<9} {:<11} {:<10}",
                    tm, sv, pn_bf, rx_bf, tx_bf
                )
            } else {
                format!("{:<9} {:<16} {:<8} {:<8} {:<8}", tm, sv, pn_bf, rx_bf, tx_bf)
            };

            buf.set_string(
                area.x + 2,
                area.y + 2 + ndx as u16,
                Self::clp(&rw, max_w),
                pn_st,
            );
        }
    }

    fn drw_ctrl(&self, f: &mut Frame, area: Rect, st: &TstSt) {
        let buf = f.buffer_mut();
        self.drw_bx(buf, area, "CONTROLS & STATUS");

        let pn_st = Style::default().fg(CLR_FG).bg(CLR_PANE_BG);
        let max_w = area.width.saturating_sub(4) as usize;

        if area.height > 2 {
            buf.set_string(area.x + 2, area.y + 1, "status: ", pn_st);

            let (st_clr, _) = match st.st.as_str() {
                "DONE" => (CLR_GRN, Modifier::BOLD),
                "ERR" | "CNCL" => (CLR_DNG, Modifier::BOLD),
                _ => (CLR_WRN, Modifier::BOLD),
            };
            let st_style = Style::default()
                .fg(st_clr)
                .bg(CLR_PANE_BG)
                .add_modifier(Modifier::BOLD);

            let mut bdg = format!("[ {} ]", st.st);
            if st.nw_rcrd && st.st == "DONE" {
                let lbl = if st.rcrd_lbl.is_empty() {
                    String::new()
                } else {
                    format!(": {}", st.rcrd_lbl)
                };
                bdg.push_str(&format!(" [ NEW RECORD{} ]", lbl));
            }
            buf.set_string(
                area.x + 10,
                area.y + 1,
                Self::clp(&bdg, area.width.saturating_sub(12) as usize),
                st_style,
            );
        }

        if area.height > 3 {
            if !st.fl_msg.is_empty() {
                let dng_st = Style::default().fg(CLR_DNG).bg(CLR_PANE_BG);
                let err_msg = format!("err: {}", st.fl_msg);
                buf.set_string(area.x + 2, area.y + 2, Self::clp(&err_msg, max_w), dng_st);
            } else if !st.rs_lnk.is_empty() {
                let url_st = Style::default()
                    .fg(CLR_ACC)
                    .bg(CLR_PANE_BG)
                    .add_modifier(Modifier::UNDERLINED | Modifier::BOLD);
                buf.set_string(area.x + 2, area.y + 2, "url: ", pn_st);
                let avail_w = area.width.saturating_sub(8) as usize;
                let u_str = if st.rs_lnk.len() <= avail_w {
                    &st.rs_lnk
                } else {
                    &st.rs_lnk[..avail_w]
                };
                buf.set_string(area.x + 7, area.y + 2, u_str, url_st);
            }
        }

        let ctrls: &[&str] = if !st.rs_lnk.is_empty() {
            &[
                "[ SPACE ] RUN / RE-RUN",
                "[   O   ] OPEN IN BROWSER",
                "[   S   ] SELECT SERVER",
                "[   C   ] STOP TEST",
                "[   L   ] CLEAR HISTORY",
                "[   Q   ] QUIT",
            ]
        } else {
            &[
                "[ SPACE ] RUN / RE-RUN",
                "[   S   ] SELECT SERVER",
                "[   C   ] STOP TEST",
                "[   L   ] CLEAR HISTORY",
                "[   Q   ] QUIT",
            ]
        };

        for (ndx, ctrl) in ctrls.iter().enumerate() {
            let py = area.y + 3 + ndx as u16;
            if py < area.y + area.height - 1 {
                buf.set_string(area.x + 2, py, Self::clp(ctrl, max_w), pn_st);
            }
        }
    }

    fn drw_rs_lnk(&self, f: &mut Frame, area: Rect, st: &TstSt) {
        let buf = f.buffer_mut();
        self.drw_bx(buf, area, "RESULT URL // SPEEDTEST.NET");

        let pn_st = Style::default().fg(CLR_FG).bg(CLR_PANE_BG);
        let url_st = Style::default()
            .fg(CLR_ACC)
            .bg(CLR_PANE_BG)
            .add_modifier(Modifier::UNDERLINED | Modifier::BOLD);
        let btn_st = Style::default()
            .fg(CLR_GRN)
            .bg(CLR_PANE_BG)
            .add_modifier(Modifier::BOLD);

        let w = area.width as usize;
        if w >= st.rs_lnk.len() + 9 {
            buf.set_string(area.x + 2, area.y + 1, "url: ", pn_st);
            buf.set_string(area.x + 7, area.y + 1, &st.rs_lnk, url_st);
        } else {
            let avail = w.saturating_sub(4);
            let u_str = if st.rs_lnk.len() <= avail {
                &st.rs_lnk
            } else {
                &st.rs_lnk[..avail]
            };
            buf.set_string(area.x + 2, area.y + 1, u_str, url_st);
        }

        let hint = "[ O: OPEN ]";
        let h_pos = area.x + area.width.saturating_sub(hint.len() as u16 + 2);
        if h_pos > area.x + 7 + st.rs_lnk.len() as u16 + 2 {
            buf.set_string(h_pos, area.y + 1, hint, btn_st);
        }
    }

    fn drw_mdl(&self, f: &mut Frame, my: u16, mx: u16, ngn: &Ngn) {
        let mw = 74.min(mx.saturating_sub(4));
        let mh = 18.min(my.saturating_sub(4));
        let wy = (my.saturating_sub(mh)) / 2;
        let wx = (mx.saturating_sub(mw)) / 2;

        let area = Rect::new(wx, wy, mw, mh);
        f.render_widget(Clear, area);

        let buf = f.buffer_mut();
        self.drw_bx(buf, area, "SELECT SERVER (OOKLA)");

        let pn_st = Style::default().fg(CLR_FG).bg(CLR_PANE_BG);
        let max_w = mw.saturating_sub(4) as usize;

        buf.set_string(
            wx + 2,
            wy + 1,
            Self::clp("UP/DOWN: NAVIGATE | ENTER: SELECT | ESC: CLOSE", max_w),
            pn_st,
        );

        let srvs = ngn.gt_srvs();
        let ttl_cnt = 1 + srvs.len();
        let vs_cnt = (mh.saturating_sub(4) as usize).max(1);
        let tp_ndx = (self.sl_ndx.saturating_sub(vs_cnt / 2)).min(ttl_cnt.saturating_sub(vs_cnt));

        for n in 0..vs_cnt {
            let ndx = tp_ndx + n;
            if ndx >= ttl_cnt {
                break;
            }

            let txt = if ndx == 0 {
                "[AUTO] CLOSEST SERVER (AUTOMATIC DETECTION)".to_string()
            } else {
                let sv = &srvs[ndx - 1];
                format!("[{}] {} - {}, {}", sv.id, sv.name, sv.location, sv.country)
            };

            let slctd = ndx == self.sl_ndx;
            let row_st = if slctd {
                Style::default()
                    .fg(CLR_MID)
                    .bg(CLR_ACC)
                    .add_modifier(Modifier::BOLD)
            } else {
                pn_st
            };

            let pd_txt = Self::pad(&txt, max_w);
            buf.set_string(wx + 2, wy + 3 + n as u16, &pd_txt, row_st);
        }
    }

    fn drw_cnft(&mut self, f: &mut Frame, my: u16, mx: u16, st: &TstSt) {
        if self.cnft_tmr <= 0 {
            return;
        }
        self.cnft_tmr -= 1;

        let glphs = ["*", "+", "\u{25c6}", "\u{25aa}", "\u{25cf}", "x", "#"];
        let buf = f.buffer_mut();

        for pt in &mut self.cnft_pts {
            if pt.lf <= 0 {
                continue;
            }
            pt.lf -= 1;
            pt.x += pt.vx + (pt.lf as f64 * 0.35).sin() * 0.28;
            pt.y += pt.vy;
            pt.vy += 0.085;
            pt.vx *= 0.985;

            let py = pt.y.round() as i32;
            let px = pt.x.round() as i32;
            if py >= 1 && py < (my as i32 - 1) && px >= 1 && px < (mx as i32 - 1) {
                let st = Style::default().fg(pt.clr).add_modifier(Modifier::BOLD);
                buf.set_string(px as u16, py as u16, glphs[pt.ch_ndx % 7], st);
            }
        }

        let lbl = if st.rcrd_lbl.is_empty() {
            String::new()
        } else {
            format!(" ({})", st.rcrd_lbl)
        };
        let bnr = format!(" [ * NEW HIGH SCORE UNLOCKED{} * ] ", lbl);
        let bx = (mx as i32 - bnr.len() as i32) / 2;
        let by = 2.max((my as i32 / 2) - 2);

        if bx > 2 && self.cnft_tmr > 10 {
            let b_clr = if (self.cnft_tmr % 16) < 8 {
                CLR_ACC
            } else {
                CLR_WRN
            };
            let b_st = Style::default()
                .fg(CLR_MID)
                .bg(b_clr)
                .add_modifier(Modifier::BOLD);
            buf.set_string(bx as u16, by as u16, &bnr, b_st);
        }
    }

    fn clp(txt: &str, mx: usize) -> &str {
        if txt.len() <= mx {
            txt
        } else {
            &txt[..mx]
        }
    }

    fn pad(txt: &str, w: usize) -> String {
        if txt.len() >= w {
            txt[..w].to_string()
        } else {
            format!("{}{}", txt, " ".repeat(w - txt.len()))
        }
    }
}
