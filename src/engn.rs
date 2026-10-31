use chrono::Local;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs::{self, File};
use std::io::{BufRead, BufReader};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Srv {
    #[serde(default)]
    pub id: i32,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub location: String,
    #[serde(default)]
    pub country: String,
    #[serde(default)]
    pub host: String,
    #[serde(default = "df_prt")]
    pub port: i32,
    #[serde(default)]
    pub ip: String,
}

fn df_prt() -> i32 {
    8080
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct HstRc {
    #[serde(default)]
    pub tm: String,
    #[serde(default)]
    pub srv: String,
    #[serde(default)]
    pub ltnc: f64,
    #[serde(default)]
    pub jttr: f64,
    #[serde(default)]
    pub dl: f64,
    #[serde(default)]
    pub ul: f64,
    #[serde(default)]
    pub lnk: String,
}

#[derive(Clone, Debug)]
pub struct TstSt {
    pub st: String,
    pub prvdr: String,
    pub nc_nm: String,
    pub rmt_ddr: String,
    pub lcl_ddr: String,
    pub srv_k: i32,
    pub srv_nm: String,
    pub srv_lc: String,
    pub srv_cntr: String,
    pub srv_hst: String,
    pub srv_prt: i32,
    pub srv_ddr: String,
    pub ltnc: f64,
    pub jttr: f64,
    pub pkt_lss: f64,
    pub rx_bw: f64,
    pub rx_byt: i64,
    pub rx_prg: f64,
    pub rx_pk: f64,
    pub rx_vg: f64,
    pub tx_bw: f64,
    pub tx_byt: i64,
    pub tx_prg: f64,
    pub tx_pk: f64,
    pub tx_vg: f64,
    pub rs_lnk: String,
    pub fl_msg: String,
    pub nw_rcrd: bool,
    pub rcrd_cnt: i32,
    pub rcrd_lbl: String,
    pub ltnc_pts: Vec<f64>,
    pub rx_pts: Vec<f64>,
    pub tx_pts: Vec<f64>,
}

impl Default for TstSt {
    fn default() -> Self {
        Self {
            st: "IDLE".to_string(),
            prvdr: String::new(),
            nc_nm: String::new(),
            rmt_ddr: String::new(),
            lcl_ddr: String::new(),
            srv_k: 0,
            srv_nm: String::new(),
            srv_lc: String::new(),
            srv_cntr: String::new(),
            srv_hst: String::new(),
            srv_prt: 0,
            srv_ddr: String::new(),
            ltnc: -1.0,
            jttr: -1.0,
            pkt_lss: -1.0,
            rx_bw: -1.0,
            rx_byt: 0,
            rx_prg: 0.0,
            rx_pk: 0.0,
            rx_vg: 0.0,
            tx_bw: -1.0,
            tx_byt: 0,
            tx_prg: 0.0,
            tx_pk: 0.0,
            tx_vg: 0.0,
            rs_lnk: String::new(),
            fl_msg: String::new(),
            nw_rcrd: false,
            rcrd_cnt: 0,
            rcrd_lbl: String::new(),
            ltnc_pts: Vec::new(),
            rx_pts: Vec::new(),
            tx_pts: Vec::new(),
        }
    }
}

impl TstSt {
    pub fn rst(&mut self) {
        self.st = "IDLE".to_string();
        self.ltnc = -1.0;
        self.jttr = -1.0;
        self.pkt_lss = -1.0;
        self.rx_bw = -1.0;
        self.rx_byt = 0;
        self.rx_prg = 0.0;
        self.rx_pk = 0.0;
        self.rx_vg = 0.0;
        self.tx_bw = -1.0;
        self.tx_byt = 0;
        self.tx_prg = 0.0;
        self.tx_pk = 0.0;
        self.tx_vg = 0.0;
        self.rs_lnk.clear();
        self.fl_msg.clear();
        self.nw_rcrd = false;
        self.rcrd_lbl.clear();
        self.ltnc_pts.clear();
        self.rx_pts.clear();
        self.tx_pts.clear();
    }
}

pub struct Ngn {
    pub bn: String,
    pub st: Arc<Mutex<TstSt>>,
    pub srv_lst: Arc<Mutex<Vec<Srv>>>,
    pub hs_sl_srv: Arc<Mutex<bool>>,
    pub sl_srv: Arc<Mutex<Option<Srv>>>,
    pub hst_f: PathBuf,
    chld: Arc<Mutex<Option<Child>>>,
}

#[derive(Deserialize)]
struct OoklaMsg {
    #[serde(rename = "type")]
    msg_tp: Option<String>,
    isp: Option<String>,
    interface: Option<OoklaIface>,
    server: Option<Srv>,
    ping: Option<OoklaPing>,
    download: Option<OoklaDlUl>,
    upload: Option<OoklaDlUl>,
    #[serde(rename = "packetLoss")]
    packet_loss: Option<f64>,
    result: Option<OoklaRs>,
    level: Option<String>,
    message: Option<String>,
}

#[derive(Deserialize)]
struct OoklaIface {
    name: Option<String>,
    #[serde(rename = "externalIp")]
    external_ip: Option<String>,
    #[serde(rename = "internalIp")]
    internal_ip: Option<String>,
}

#[derive(Deserialize)]
struct OoklaPing {
    jitter: Option<f64>,
    latency: Option<f64>,
}

#[derive(Deserialize)]
struct OoklaDlUl {
    bandwidth: Option<f64>,
    bytes: Option<i64>,
    progress: Option<f64>,
}

#[derive(Deserialize)]
struct OoklaRs {
    url: Option<String>,
}

#[derive(Deserialize)]
struct OoklaSrvsMsg {
    servers: Option<Vec<Srv>>,
}

fn gt_dr() -> PathBuf {
    if let Ok(exe) = env::current_exe() {
        if let Some(p) = exe.parent() {
            return p.to_path_buf();
        }
    }
    PathBuf::from(".")
}

impl Ngn {
    pub fn new(bn_pth: Option<String>) -> Self {
        let hm = env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let hst_f = PathBuf::from(hm).join(".speedtest_tui_hist.json");

        let bn = bn_pth.unwrap_or_else(|| Self::fnd_bn());

        Self {
            bn,
            st: Arc::new(Mutex::new(TstSt::default())),
            srv_lst: Arc::new(Mutex::new(Vec::new())),
            hs_sl_srv: Arc::new(Mutex::new(false)),
            sl_srv: Arc::new(Mutex::new(None)),
            hst_f,
            chld: Arc::new(Mutex::new(None)),
        }
    }

    pub fn fnd_bn() -> String {
        let dr = gt_dr();
        let mut pths = vec![
            dr.join("bin").join("speedtest"),
            PathBuf::from("./bin/speedtest"),
            PathBuf::from("/usr/bin/speedtest"),
            PathBuf::from("/usr/local/bin/speedtest"),
        ];
        if let Ok(hm) = env::var("HOME") {
            pths.push(PathBuf::from(&hm).join(".local/bin/speedtest-ookla"));
            pths.push(PathBuf::from(&hm).join(".local/bin/speedtest"));
            pths.push(PathBuf::from(&hm).join(".local/share/speedtui/bin/speedtest"));
        }

        for pth in pths {
            if pth.is_file() {
                if let Ok(out) = Command::new(&pth).arg("--version").output() {
                    let txt = String::from_utf8_lossy(&out.stdout);
                    let err = String::from_utf8_lossy(&out.stderr);
                    if txt.contains("Ookla") || err.contains("Ookla") {
                        return pth.to_string_lossy().to_string();
                    }
                }
            }
        }

        Self::dwnld_bn()
    }

    pub fn dwnld_bn() -> String {
        let local_bndr = gt_dr().join("bin");
        let bndr = if fs::create_dir_all(&local_bndr).is_ok() {
            local_bndr
        } else if let Ok(hm) = env::var("HOME") {
            let u_dir = PathBuf::from(hm).join(".local/share/speedtui/bin");
            let _ = fs::create_dir_all(&u_dir);
            u_dir
        } else {
            PathBuf::from("/tmp")
        };

        let tgt = bndr.join("speedtest");
        if tgt.is_file() {
            return tgt.to_string_lossy().to_string();
        }

        let lnk = "https://install.speedtest.net/app/cli/ookla-speedtest-1.2.0-linux-x86_64.tgz";
        let cmd = format!(
            "curl -fsSL \"{}\" | tar -xz -C \"{}\" 2>/dev/null",
            lnk,
            bndr.display()
        );
        let st = Command::new("sh").arg("-c").arg(&cmd).status();
        if let Ok(s) = st {
            if s.success() && tgt.is_file() {
                if let Ok(meta) = fs::metadata(&tgt) {
                    let mut perms = meta.permissions();
                    perms.set_mode(0o755);
                    let _ = fs::set_permissions(&tgt, perms);
                }
                return tgt.to_string_lossy().to_string();
            }
        }
        String::new()
    }

    #[allow(dead_code)]
    pub fn ld_srvs(&self) -> Vec<Srv> {
        if self.bn.is_empty() || !Path::new(&self.bn).is_file() {
            return Vec::new();
        }

        let out = Command::new(&self.bn)
            .args(["-L", "--format=json", "--accept-license", "--accept-gdpr"])
            .output();

        let mut res = Vec::new();
        if let Ok(o) = out {
            let txt = String::from_utf8_lossy(&o.stdout);
            if let Ok(msg) = serde_json::from_str::<OoklaSrvsMsg>(&txt) {
                if let Some(srvs) = msg.servers {
                    res = srvs.into_iter().filter(|s| s.id > 0).collect();
                }
            }
        }

        if let Ok(mut lck) = self.srv_lst.lock() {
            *lck = res.clone();
        }
        res
    }

    pub fn ld_srvs_bg(&self) {
        let bn = self.bn.clone();
        let srv_lst = Arc::clone(&self.srv_lst);
        thread::spawn(move || {
            if bn.is_empty() || !Path::new(&bn).is_file() {
                return;
            }
            if let Ok(o) = Command::new(&bn)
                .args(["-L", "--format=json", "--accept-license", "--accept-gdpr"])
                .output()
            {
                let txt = String::from_utf8_lossy(&o.stdout);
                if let Ok(msg) = serde_json::from_str::<OoklaSrvsMsg>(&txt) {
                    if let Some(srvs) = msg.servers {
                        let res: Vec<Srv> = srvs.into_iter().filter(|s| s.id > 0).collect();
                        if let Ok(mut lck) = srv_lst.lock() {
                            *lck = res;
                        }
                    }
                }
            }
        });
    }

    pub fn strt(&self, srv_k: i32) -> bool {
        {
            let mut st = match self.st.lock() {
                Ok(s) => s,
                Err(_) => return false,
            };
            if st.st == "PING" || st.st == "DL" || st.st == "UL" {
                return false;
            }
            st.rst();
            if self.bn.is_empty() || !Path::new(&self.bn).is_file() {
                st.st = "ERR".to_string();
                st.fl_msg = "BIN NOT FOUND".to_string();
                return false;
            }
            st.st = "PING".to_string();
        }

        let mut tgt_k = srv_k;
        if tgt_k <= 0 {
            if let Ok(hs) = self.hs_sl_srv.lock() {
                if *hs {
                    if let Ok(sv_lck) = self.sl_srv.lock() {
                        if let Some(ref sv) = *sv_lck {
                            tgt_k = sv.id;
                        }
                    }
                }
            }
        }

        let mut cmd = Command::new(&self.bn);
        cmd.args(["-f", "jsonl", "--accept-license", "--accept-gdpr"]);
        if tgt_k > 0 {
            cmd.args(["-s", &tgt_k.to_string()]);
        }
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        let mut chld = match cmd.spawn() {
            Ok(c) => c,
            Err(e) => {
                if let Ok(mut st) = self.st.lock() {
                    st.st = "ERR".to_string();
                    st.fl_msg = format!("SPAWN ERR: {}", e);
                }
                return false;
            }
        };

        let stdout = chld.stdout.take();
        if let Ok(mut c_lck) = self.chld.lock() {
            *c_lck = Some(chld);
        }

        let st_ref = Arc::clone(&self.st);
        let chld_ref = Arc::clone(&self.chld);
        let hst_f = self.hst_f.clone();

        thread::spawn(move || {
            if let Some(out) = stdout {
                let rdr = BufReader::new(out);
                for ln in rdr.lines().map_while(Result::ok) {
                    let tr = ln.trim();
                    if !tr.is_empty() {
                        Self::prsc_ln(tr, &st_ref, &hst_f);
                    }
                }
            }

            let mut exit_code = 0;
            if let Ok(mut c_lck) = chld_ref.lock() {
                if let Some(mut c) = c_lck.take() {
                    if let Ok(st) = c.wait() {
                        exit_code = st.code().unwrap_or(0);
                    }
                }
            }

            if let Ok(mut st) = st_ref.lock() {
                if exit_code != 0 && st.st != "CNCL" && st.st != "DONE" {
                    st.st = "ERR".to_string();
                    if st.fl_msg.is_empty() {
                        st.fl_msg = format!("RC {}", exit_code);
                    }
                }
            }
        });

        true
    }

    pub fn stp(&self) {
        if let Ok(mut st) = self.st.lock() {
            if st.st == "PING" || st.st == "DL" || st.st == "UL" {
                st.st = "CNCL".to_string();
            }
        }
        if let Ok(mut c_lck) = self.chld.lock() {
            if let Some(ref mut c) = *c_lck {
                let pid = c.id() as i32;
                if pid > 0 {
                    unsafe {
                        libc::kill(pid, libc::SIGTERM);
                    }
                }
            }
        }
    }

    fn prsc_ln(ln: &str, st_ref: &Arc<Mutex<TstSt>>, hst_f: &Path) {
        let msg: OoklaMsg = match serde_json::from_str(ln) {
            Ok(m) => m,
            Err(_) => return,
        };

        let tp = match msg.msg_tp {
            Some(t) => t,
            None => return,
        };

        let mut hst_rc = HstRc::default();
        let mut sv_flg = false;

        {
            let mut st = match st_ref.lock() {
                Ok(s) => s,
                Err(_) => return,
            };

            if st.st == "CNCL" {
                return;
            }

            match tp.as_str() {
                "testStart" => {
                    st.prvdr = msg.isp.unwrap_or_default();
                    if let Some(iface) = msg.interface {
                        st.nc_nm = iface.name.unwrap_or_default();
                        st.rmt_ddr = iface.external_ip.unwrap_or_default();
                        st.lcl_ddr = iface.internal_ip.unwrap_or_default();
                    }
                    if let Some(sv) = msg.server {
                        st.srv_k = sv.id;
                        st.srv_nm = sv.name;
                        st.srv_lc = sv.location;
                        st.srv_cntr = sv.country;
                        st.srv_hst = sv.host;
                        st.srv_prt = sv.port;
                        st.srv_ddr = sv.ip;
                    }
                    st.st = "PING".to_string();
                }
                "ping" => {
                    st.st = "PING".to_string();
                    if let Some(png) = msg.ping {
                        if let Some(lt) = png.latency {
                            if lt >= 0.0 {
                                st.ltnc = lt;
                                st.ltnc_pts.push(lt);
                            }
                        }
                        if let Some(jt) = png.jitter {
                            if jt >= 0.0 {
                                st.jttr = jt;
                            }
                        }
                    }
                }
                "download" => {
                    st.st = "DL".to_string();
                    if let Some(rx) = msg.download {
                        let bw = rx.bandwidth.unwrap_or(0.0);
                        st.rx_bw = bw;
                        st.rx_byt = rx.bytes.unwrap_or(0);
                        st.rx_prg = rx.progress.unwrap_or(0.0);
                        let mbps = (bw * 8.0) / 1_000_000.0;
                        st.rx_pts.push(mbps);
                        if mbps > st.rx_pk {
                            st.rx_pk = mbps;
                        }
                        if !st.rx_pts.is_empty() {
                            let sm: f64 = st.rx_pts.iter().sum();
                            st.rx_vg = sm / st.rx_pts.len() as f64;
                        }
                    }
                }
                "upload" => {
                    st.st = "UL".to_string();
                    if let Some(tx) = msg.upload {
                        let bw = tx.bandwidth.unwrap_or(0.0);
                        st.tx_bw = bw;
                        st.tx_byt = tx.bytes.unwrap_or(0);
                        st.tx_prg = tx.progress.unwrap_or(0.0);
                        let mbps = (bw * 8.0) / 1_000_000.0;
                        st.tx_pts.push(mbps);
                        if mbps > st.tx_pk {
                            st.tx_pk = mbps;
                        }
                        if !st.tx_pts.is_empty() {
                            let sm: f64 = st.tx_pts.iter().sum();
                            st.tx_vg = sm / st.tx_pts.len() as f64;
                        }
                    }
                }
                "result" => {
                    st.st = "DONE".to_string();
                    if let Some(png) = msg.ping {
                        st.ltnc = png.latency.unwrap_or(st.ltnc);
                        st.jttr = png.jitter.unwrap_or(st.jttr);
                    }
                    st.pkt_lss = msg.packet_loss.unwrap_or(0.0);
                    if let Some(rx) = msg.download {
                        if let Some(bw) = rx.bandwidth {
                            if bw >= 0.0 {
                                st.rx_bw = bw;
                            }
                        }
                    }
                    if let Some(tx) = msg.upload {
                        if let Some(bw) = tx.bandwidth {
                            if bw >= 0.0 {
                                st.tx_bw = bw;
                            }
                        }
                    }
                    if let Some(rs) = msg.result {
                        st.rs_lnk = rs.url.unwrap_or_default();
                    }

                    let tm_str = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
                    hst_rc.tm = tm_str;
                    hst_rc.srv = format!("{} ({})", st.srv_nm, st.srv_lc);
                    hst_rc.ltnc = if st.ltnc >= 0.0 { st.ltnc } else { 0.0 };
                    hst_rc.jttr = if st.jttr >= 0.0 { st.jttr } else { 0.0 };
                    hst_rc.dl = if st.rx_bw >= 0.0 {
                        st.rx_bw * 8.0 / 1_000_000.0
                    } else {
                        0.0
                    };
                    hst_rc.ul = if st.tx_bw >= 0.0 {
                        st.tx_bw * 8.0 / 1_000_000.0
                    } else {
                        0.0
                    };
                    hst_rc.lnk = st.rs_lnk.clone();

                    let prv_lst = Self::ld_hst_from(hst_f);
                    let mut mx_rx = 0.0_f64;
                    let mut mx_tx = 0.0_f64;
                    let mut mn_lt = 999_999.0_f64;
                    for r in &prv_lst {
                        if r.dl > mx_rx {
                            mx_rx = r.dl;
                        }
                        if r.ul > mx_tx {
                            mx_tx = r.ul;
                        }
                        if r.ltnc > 0.0 && r.ltnc < mn_lt {
                            mn_lt = r.ltnc;
                        }
                    }

                    let bt_rx = hst_rc.dl > 0.0 && (prv_lst.is_empty() || hst_rc.dl > mx_rx);
                    let bt_tx = hst_rc.ul > 0.0 && (prv_lst.is_empty() || hst_rc.ul > mx_tx);
                    let bt_lt = hst_rc.ltnc > 0.0 && !prv_lst.is_empty() && hst_rc.ltnc < mn_lt;

                    if bt_rx || bt_tx || bt_lt {
                        st.nw_rcrd = true;
                        st.rcrd_cnt += 1;
                        let mut lbl = String::new();
                        if bt_rx {
                            lbl.push_str("DL ");
                        }
                        if bt_tx {
                            lbl.push_str("UL ");
                        }
                        if bt_lt && !bt_rx && !bt_tx {
                            lbl.push_str("PING ");
                        }
                        st.rcrd_lbl = lbl.trim_end().to_string();
                    }
                    sv_flg = true;
                }
                "log" => {
                    if let Some(lvl) = msg.level {
                        if lvl == "error" {
                            st.fl_msg = msg.message.unwrap_or_default();
                        }
                    }
                }
                _ => {}
            }
        }

        if sv_flg {
            Self::sv_hst_to(hst_f, &hst_rc);
        }
    }

    pub fn gt_st(&self) -> TstSt {
        self.st.lock().map(|s| s.clone()).unwrap_or_default()
    }

    pub fn gt_srvs(&self) -> Vec<Srv> {
        self.srv_lst.lock().map(|s| s.clone()).unwrap_or_default()
    }

    pub fn st_sl_srv(&self, sv: Option<Srv>) {
        if let Ok(mut hs) = self.hs_sl_srv.lock() {
            *hs = sv.is_some();
        }
        if let Ok(mut sl) = self.sl_srv.lock() {
            *sl = sv;
        }
    }

    pub fn ld_hst(&self) -> Vec<HstRc> {
        Self::ld_hst_from(&self.hst_f)
    }

    fn ld_hst_from(pth: &Path) -> Vec<HstRc> {
        if let Ok(fp) = File::open(pth) {
            serde_json::from_reader(fp).unwrap_or_default()
        } else {
            Vec::new()
        }
    }

    fn sv_hst_to(pth: &Path, rc: &HstRc) {
        let mut lst = Self::ld_hst_from(pth);
        lst.insert(0, rc.clone());
        if lst.len() > 30 {
            lst.truncate(30);
        }
        if let Ok(fp) = File::create(pth) {
            let _ = serde_json::to_writer_pretty(fp, &lst);
        }
    }

    pub fn clr_hst(&self) {
        let _ = fs::remove_file(&self.hst_f);
    }
}

impl Drop for Ngn {
    fn drop(&mut self) {
        self.stp();
    }
}
