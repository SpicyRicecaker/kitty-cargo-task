use clap::Parser;
use serde::{Deserialize, Serialize};
use serde_json::{Result, Value};
use std::{process::Command, thread::sleep, time::Duration};

#[derive(Debug, Clone)]
struct Package {
    i_current_tab: usize,
    tabs: Vec<Tab>,
    i_current_window_cwd: usize,
    windows_cwd: Vec<WindowCWD>,
    window_current_cwd: String,
    id_window_current: usize
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Screen {
    id: usize,
    tabs: Vec<Tab>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Tab {
    id: usize,
    windows: Vec<Window>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
// note: fairly certain that cwd for window is bugged
struct Window {
    id: usize,
    is_self: bool,
    foreground_processes: Vec<ForegroundProcess>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ForegroundProcess {
    cmdline: Vec<String>,
    cwd: String,
}

fn kitty_ls() -> String {
    let output = Command::new("kitty")
        .args(["@", "ls"])
        .output()
        .expect("failed to run kitty");
    str::from_utf8(&output.stdout).unwrap().to_string()
}

fn kitty_get_windows_package(kitty_ls: &str) -> Package {
    let screens: Vec<Screen> = serde_json::from_str(kitty_ls).expect("failed to parse");

    // assume only 1 screen
    let tabs = screens[0].tabs.clone();
    // dbg!(&tabs);
    let (i_current_tab, window_current) = tabs
        .iter()
        .enumerate()
        .flat_map(|(i_t, t)| t.windows.iter().map(move |w| (i_t, w)))
        .find(|(_, w)| w.is_self)
        .unwrap();
    let window_current_cwd = window_current
        .foreground_processes
        .iter()
        .rev()
        .next()
        .unwrap()
        .cwd
        .to_string();
    let id_window_current = window_current.id;

    // task 1:
    // get a list of all windows
    //   with shell
    //   with cwd equivalent to self
    //   OR self
    // task 2:
    // get the index of window which is self

    // task 1 --
    // get list of all windows with shell or is self
    // find self, get cwd of self (most recent foreground process)
    // filter all shell windows by cwd
    // find i_self
    //
    // alternatively, we could
    // find self
    // get cwd
    // get list of all windows with shell with cwd equal to self (most recent foreground process) or self
    // find self

    let windows_cwd = tabs
        .iter()
        .enumerate()
        .flat_map(|(i_tab, t)| t.windows.iter().map(move |w| (i_tab, w)))
        .filter_map(|(i_tab, w)| {
            if w.is_self {
                Some(WindowCWD { id: w.id, i_tab })
            } else if let Some(fp) = w.foreground_processes.iter().find(|fp| {
                fp.cmdline.iter().any(|cmd| {
                    ["zsh", "bash", "fish", "sh", "nu", "ksh"]
                        .iter()
                        .any(|shell| cmd.contains(shell))
                })
            }) && fp.cwd == window_current_cwd
            {
                Some(WindowCWD { id: w.id, i_tab })
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    let i_current_window_cwd = windows_cwd
        .iter()
        .position(|w| w.id == window_current.id)
        .unwrap();

    Package {
        i_current_tab,
        i_current_window_cwd,
        tabs,
        windows_cwd,
        window_current_cwd,
        id_window_current
    }
}

#[test]
fn test_kitty_get_windows_package() {
    assert_eq!(
        &hash_file("test.txt").unwrap(),
        "9d3a91ef65132ed9d057ad920599d5c1341dc032ea3f724a64b0f6fabd542e30"
    );

    let kitty_ls = std::fs::read_to_string("test.txt").unwrap();

    dbg!(kitty_get_windows_package(&kitty_ls));
}

use sha2::{Digest, Sha256};
use std::{fs, io};

fn hash_file(path: &str) -> io::Result<String> {
    let mut file = fs::File::open(path)?;
    let mut hasher = Sha256::new();

    // Efficiently stream the file into the hasher
    io::copy(&mut file, &mut hasher)?;

    let hash = hasher.finalize();
    Ok(format!("{:x}", hash))
}

#[derive(Debug, Clone)]
struct WindowCWD {
    id: usize,
    i_tab: usize,
}

#[derive(Debug, Clone)]
struct WindowCWDDiff {
    id: usize,
    dist: usize,
}

// find index of right window rel to self (if any)
// dx = take (length - 1 + 1) - index of right window to find num spaces we need to move new window
//      or 0 if no space
// create new window
// move window back by dx indices
fn kitty_get_needed_dx_new_tab_to_right_of_current_tab(
    i_current_tab: usize,
    tabs: &[Tab],
) -> usize {
    let tab_right = tabs.get(i_current_tab + 1);

    if tab_right.is_some() {
        tabs.len() - (i_current_tab + 1)
    } else {
        0
    }
}

#[cfg(test)]
mod test_get_needed_dx_new_tab_to_right_of_current_tab {
    use super::*;
    #[test]
    fn test1() {
        let i_current_tab = 0;
        let tabs = vec![
            Tab {
                id: 0,
                windows: vec![],
            },
            Tab {
                id: 1,
                windows: vec![],
            },
        ];

        let dx = kitty_get_needed_dx_new_tab_to_right_of_current_tab(i_current_tab, &tabs);
        assert_eq!(dx, 1);
    }

    #[test]
    fn test2() {
        let i_current_tab = 1;
        let tabs = vec![
            Tab {
                id: 0,
                windows: vec![],
            },
            Tab {
                id: 1, // <---------
                windows: vec![],
            },
            Tab {
                id: 2,
                windows: vec![],
            },
            Tab {
                id: 3,
                windows: vec![],
            },
        ];

        let dx = kitty_get_needed_dx_new_tab_to_right_of_current_tab(i_current_tab, &tabs);
        assert_eq!(dx, 2);
    }

    #[test]
    fn test3() {
        let i_current_tab = 0;
        let tabs = vec![Tab {
            id: 0, // <---------
            windows: vec![],
        }];

        let dx = kitty_get_needed_dx_new_tab_to_right_of_current_tab(i_current_tab, &tabs);
        assert_eq!(dx, 0);
    }

    #[test]
    fn test_random_id() {
        let i_current_tab = 1;
        let tabs = vec![
            Tab {
                id: 99,
                windows: vec![],
            },
            Tab {
                id: 420, // <---------
                windows: vec![],
            },
            Tab {
                id: 69,
                windows: vec![],
            },
            Tab {
                id: 67,
                windows: vec![],
            },
        ];

        let dx = kitty_get_needed_dx_new_tab_to_right_of_current_tab(i_current_tab, &tabs);
        assert_eq!(dx, 2);
    }
}

fn kitty_get_id_closest_window_with_cwd(
    i_current_window: usize,
    windows: &[WindowCWD],
) -> Option<usize> {
    let i_current_tab = windows[i_current_window].i_tab;
    let windows = windows
        .into_iter()
        .enumerate()
        .map(|(i, a)| WindowCWDDiff {
            id: a.id,
            dist: i.abs_diff(i_current_tab),
        })
        .collect::<Vec<_>>();

    let i_l: Option<usize> = i_current_window.checked_sub(1);
    let i_r: Option<usize> = i_current_window.checked_add(1);

    let mut l_closest = usize::MAX;
    let mut r_closest = usize::MAX;

    if let Some(i_l) = i_l
        && i_l < i_current_window
    {
        l_closest = windows[i_l as usize].dist;
    }

    if let Some(i_r) = i_r
        && i_r < windows.len()
    {
        r_closest = windows[i_r as usize].dist;
    }

    let i_closest_window: Option<usize> = match (l_closest, r_closest) {
        (usize::MAX, usize::MAX) => None,
        (_, usize::MAX) => i_l,
        (usize::MAX, _) | (_, _) => i_r,
    };

    if let Some(i_closest_window) = i_closest_window {
        Some(windows[i_closest_window as usize].id)
    } else {
        None
    }
}

#[cfg(test)]
mod test_get_id_closest_window_with_cwd {
    use super::*;

    #[test]
    fn test1() {
        let i_current_window = 0;
        let windows = vec![
            WindowCWD {
                id: 0,
                i_tab: 0,
            },
            WindowCWD {
                id: 1,
                i_tab: 1,
            },
        ];
        assert_eq!(
            kitty_get_id_closest_window_with_cwd(i_current_window, &windows).unwrap(),
            1
        );
    }

    #[test]
    fn test2() {
        let i_current_window = 0;
        let windows = vec![WindowCWD {
            id: 0,
            i_tab: 0
        }];
        assert_eq!(
            kitty_get_id_closest_window_with_cwd(i_current_window, &windows),
            None
        );
    }

    #[test]
    fn test3() {
        let i_current_window = 1;
        let windows = vec![
            WindowCWD {
                id: 0,
                i_tab: 0
            },
            WindowCWD {
                id: 1,
                i_tab: 1
            },
            WindowCWD {
                id: 2,
                i_tab: 2
            },
            WindowCWD {
                id: 3,
                i_tab: 3
            },
        ];
        assert_eq!(
            kitty_get_id_closest_window_with_cwd(i_current_window, &windows).unwrap(),
            2
        );
    }

    #[test]
    fn test4() {
        let i_current_window = 1;
        let windows = vec![
            WindowCWD {
                id: 0,
                i_tab: 0,
            },
            WindowCWD {
                id: 1,
                i_tab: 1,
            },
            WindowCWD {
                id: 2,
                i_tab: 2,
            },
            WindowCWD {
                id: 3,
                i_tab: 3,
            },
        ];
        assert_eq!(
            kitty_get_id_closest_window_with_cwd(i_current_window, &windows).unwrap(),
            2
        );
    }

    #[test]
    fn test5() {
        let i_current_window = 2;
        let windows = vec![
            WindowCWD {
                id: 0,
                i_tab: 0,
            },
            WindowCWD {
                id: 1,
                i_tab: 1,
            },
            WindowCWD {
                id: 2,
                i_tab: 2,
            },
            WindowCWD {
                id: 3,
                i_tab: 3,
            },
            WindowCWD {
                id: 4,
                i_tab: 4,
            },
        ];
        assert_eq!(
            kitty_get_id_closest_window_with_cwd(i_current_window, &windows).unwrap(),
            3
        );
    }
}

fn kitty_move_focused_back_by(dx: usize) {
    // dbg!(format!("moving back by {dx}"));
    let _ = Command::new("kitty")
        .args(["@", "kitten", "mykitten123.py", &format!("{dx}")])
        .output()
        .expect("failed to focus tab");
}

fn kitty_focus_window(id: usize) {
    let _ = Command::new("kitty")
        .args(["@", "focus-window", "-m", &format!("id:{id}")])
        .output()
        .expect("failed to focus tab");
}

fn kitty_new_tab(cwd: &str, dont_take_focus: bool) {
    let mut args = vec!["@", "launch", "--hold=true", "--type=tab", "--cwd", cwd];
    if dont_take_focus {
        args.push("--dont-take-focus");
    }
    let _ = Command::new("kitty")
        .args(args)
        .output()
        .expect("failed to launch tab");
}

fn kitty_send_cmd(id: isize, cmd: &str) {
    let _ = Command::new("kitty")
        .args([
            "@",
            "send-text",
            "-m",
            &format!("id:{id}"),
            &format!("{cmd}\\r"),
        ])
        .output()
        .expect("failed to run cargo");
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Flags {
    /// whether or not to focus the new tab
    #[arg(short, long)]
    dont_take_focus: bool,
    /// (wip) whether or not to return to original tab after command is run.  
    /// currently replicates dont_take_focus behavior
    #[arg(short, long)]
    jump_back: bool,
    /// whether or not to open tab to the right next to the currently focused tab (if applicable)
    #[arg(short, long)]
    adjacent: bool,
    /// command, if any, to run in the new tab
    #[arg(short, long)]
    command: Option<String>,
}

fn main() {
    let kitty_ls = kitty_ls();
    let package = kitty_get_windows_package(&kitty_ls);
    // dbg!(&package.windows);

    let flags = Flags::parse();

    let id_window_runner: isize = if let Some(id_window) =
        kitty_get_id_closest_window_with_cwd(package.i_current_window_cwd, &package.windows_cwd)
    {
        // println!("dont_take_focusing window {id_window_runner}");
        if !flags.dont_take_focus {
            kitty_focus_window(id_window);
        }
        id_window as isize
    } else {
        // println!("launching new tab");
        // select window
        kitty_new_tab(&package.window_current_cwd, flags.dont_take_focus);
        -1 as isize
    };
    // dbg!(&package.windows_cwd);
    if flags.adjacent && id_window_runner == -1 {
        let dx = kitty_get_needed_dx_new_tab_to_right_of_current_tab(
            package.i_current_tab,
            &package.tabs,
        );
        kitty_move_focused_back_by(dx);
    }
    if let Some(cmd) = flags.command {
        kitty_send_cmd(id_window_runner, &cmd);
    }
    if flags.jump_back {
        kitty_focus_window(package.id_window_current);
    }
}
