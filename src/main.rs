mod dsp;
mod engn;
mod grph;

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use dsp::Dsp;
use engn::Ngn;
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io::{stdout, Result};
use std::panic;
use std::process::{Command, Stdio};
use std::time::Duration;

fn main() -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut out = stdout();
    execute!(out, EnterAlternateScreen)?;
    let bck = CrosstermBackend::new(out);
    let mut trm = Terminal::new(bck)?;
    trm.hide_cursor()?;

    // Restore terminal on panic
    let orig_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        let _ = disable_raw_mode();
        let _ = execute!(stdout(), LeaveAlternateScreen);
        orig_hook(panic_info);
    }));

    let ngn = Ngn::new(None);
    ngn.ld_srvs_bg();

    let mut dsp = Dsp::new();

    loop {
        trm.draw(|f| {
            dsp.rndr(f, &ngn);
        })?;

        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(k) = event::read()? {
                if k.kind == KeyEventKind::Press {
                    match k.code {
                        KeyCode::Char('q') | KeyCode::Char('Q') => {
                            if dsp.shw_mdl {
                                dsp.shw_mdl = false;
                            } else {
                                ngn.stp();
                                break;
                            }
                        }
                        KeyCode::Esc => {
                            if dsp.shw_mdl {
                                dsp.shw_mdl = false;
                            }
                        }
                        KeyCode::Char('s') | KeyCode::Char('S') => {
                            if !dsp.shw_mdl {
                                if ngn.gt_srvs().is_empty() {
                                    ngn.ld_srvs_bg();
                                }
                                dsp.shw_mdl = true;
                                dsp.sl_ndx = 0;
                            } else {
                                dsp.shw_mdl = false;
                            }
                        }
                        KeyCode::Char('c') | KeyCode::Char('C') => {
                            ngn.stp();
                        }
                        KeyCode::Char('l') | KeyCode::Char('L') => {
                            ngn.clr_hst();
                        }
                        KeyCode::Char('o') | KeyCode::Char('O') => {
                            let st = ngn.gt_st();
                            if !st.rs_lnk.is_empty() {
                                let _ = Command::new("xdg-open")
                                    .arg(&st.rs_lnk)
                                    .stdout(Stdio::null())
                                    .stderr(Stdio::null())
                                    .spawn();
                            }
                        }
                        KeyCode::Char(' ') | KeyCode::Enter => {
                            if dsp.shw_mdl {
                                let srvs = ngn.gt_srvs();
                                if dsp.sl_ndx == 0 {
                                    ngn.st_sl_srv(None);
                                } else if dsp.sl_ndx - 1 < srvs.len() {
                                    ngn.st_sl_srv(Some(srvs[dsp.sl_ndx - 1].clone()));
                                }
                                dsp.shw_mdl = false;
                            } else {
                                ngn.strt(0);
                            }
                        }
                        KeyCode::Up => {
                            if dsp.shw_mdl && dsp.sl_ndx > 0 {
                                dsp.sl_ndx -= 1;
                            }
                        }
                        KeyCode::Down => {
                            if dsp.shw_mdl {
                                let ttl_cnt = 1 + ngn.gt_srvs().len();
                                if dsp.sl_ndx < ttl_cnt.saturating_sub(1) {
                                    dsp.sl_ndx += 1;
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    // Teardown terminal
    disable_raw_mode()?;
    execute!(trm.backend_mut(), LeaveAlternateScreen)?;
    trm.show_cursor()?;

    Ok(())
}

