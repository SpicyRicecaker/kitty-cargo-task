use std::{process::Command, thread::sleep, time::Duration};
use serde_json::{Result, Value};

#[derive(Debug)]
struct Package {
    i_current_window: usize,
    windows: Vec<WindowA>
}
fn process() -> Package { let output = Command::new("kitty").args(["@", "ls"]).output().expect("failed to run kitty");
    let kitty_ls = str::from_utf8(&output.stdout).unwrap();
    let screens: Value = serde_json::from_str(kitty_ls).expect("failed to parse");
    // dbg!(&v);

    let mut windows = vec![];
    let mut i_current_window = None;
    for screen in screens.as_array().unwrap().iter() {
        for tab in screen["tabs"].as_array().unwrap().iter() {
            'a: for window in tab["windows"].as_array().unwrap().iter() {
                for foreground_process in window["foreground_processes"].as_array().unwrap().iter() {
                    if window["is_self"].as_bool().unwrap() {
                        windows.push(WindowA { id: window["id"].as_u64().unwrap() as usize, cwd: foreground_process["cwd"].as_str().unwrap().into() }); // use foreground over window cwd
                        i_current_window = Some(windows.len() - 1);
                        continue 'a;
                    } else {
                        // dbg!(foreground_process["cmdline"].as_array().unwrap());
                        // only add zsh windows as jump options
                        for cmd in foreground_process["cmdline"].as_array().unwrap().iter() {
                            let cmd = cmd.as_str().unwrap();
                            if ["zsh", "bash", "fish", "sh", "nu", "ksh"].iter().any(|shell| cmd.contains(shell)) {
                                // dbg!("gg contains");
                                windows.push(WindowA { id: window["id"].as_u64().unwrap() as usize, cwd: foreground_process["cwd"].as_str().unwrap().into() }); // use foreground over window cwd
                                continue 'a;
                            }
                        }
                    }
                }
            }
        }
        break;
    }
    // dbg!(&windows);

    Package {
        i_current_window: i_current_window.unwrap(),
        windows
    }
}

#[derive(Debug)]
struct WindowA {
    id: usize,
    cwd: String,
}

#[derive(Debug)]
struct WindowB {
    id: usize,
    dist: usize,
    cwd: String,
}

fn choose(i_current_window: usize, windows: Vec<WindowA>) -> Option<usize> {
    let (current_window_id, current_window_cwd) = {
        let t = &windows[i_current_window];
        (t.id, t.cwd.clone())
    };

    let windows = windows
        .into_iter()
        .enumerate()
        .map(|(i, a)| WindowB {
            id: a.id,
            cwd: a.cwd,
            dist: i.abs_diff(i_current_window),
        })
        .filter(|w| w.cwd == current_window_cwd)
        .collect::<Vec<_>>();

    let i_current_window: usize = windows
        .iter()
        .enumerate()
        .find(|(_, w)| w.id == current_window_id)
        .unwrap()
        .0;

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

#[test]
fn test1() {
    let i_current_window = 0;
    let windows = vec![
        WindowA {
            id: 0,
            cwd: "cargo".into(),
        },
        WindowA {
            id: 1,
            cwd: "cargo".into(),
        },
    ];
    assert_eq!(choose(i_current_window, windows).unwrap(), 1);
}

#[test]
fn test2() {
    let i_current_window = 0;
    let windows = vec![WindowA {
        id: 0,
        cwd: "cargo".into(),
    }];
    assert_eq!(choose(i_current_window, windows), None);
}

#[test]
fn test3() {
    let i_current_window = 1;
    let windows = vec![
        WindowA {
            id: 0,
            cwd: "cargo".into(),
        },
        WindowA {
            id: 1,
            cwd: "cargo".into(),
        },
        WindowA {
            id: 2,
            cwd: "cargo".into(),
        },
        WindowA {
            id: 3,
            cwd: "cargo".into(),
        },
    ];
    assert_eq!(choose(i_current_window, windows).unwrap(), 2);
}

#[test]
fn test4() {
    let i_current_window = 1;
    let windows = vec![
        WindowA {
            id: 0,
            cwd: "cargo".into(),
        },
        WindowA {
            id: 1,
            cwd: "cargo".into(),
        },
        WindowA {
            id: 2,
            cwd: "lol".into(),
        },
        WindowA {
            id: 3,
            cwd: "cargo".into(),
        },
    ];
    assert_eq!(choose(i_current_window, windows).unwrap(), 3);
}

#[test]
fn test5() {
    let i_current_window = 2;
    let windows = vec![
        WindowA {
            id: 0,
            cwd: "cargo".into(),
        },
        WindowA {
            id: 1,
            cwd: "lol".into(),
        },
        WindowA {
            id: 2,
            cwd: "cargo".into(),
        },
        WindowA {
            id: 2,
            cwd: "lol".into(),
        },
        WindowA {
            id: 3,
            cwd: "lol".into(),
        },
    ];
    assert_eq!(choose(i_current_window, windows).unwrap(), 0);
}

fn focus_window(id: usize) {
    let output = Command::new("kitty").args(["@", "focus-window", "-m", &format!("id:{id}")]).output().expect("failed to focus tab");
}

fn new_tab(cwd: &str, dont_take_focus: bool) {
    let mut args = vec!["@", "launch", "--hold=true", "--type=tab", "--cwd", cwd];
    if dont_take_focus {
        args.push("--dont-take-focus");
    }
    let output = Command::new("kitty").args(args).output().expect("failed to launch tab");
}

fn cargo(id: isize) {
    let output = Command::new("kitty").args(["@", "send-text", "-m", &format!("id:{id}"), "cargo run\\r"]).output().expect("failed to run cargo");
}

struct Flags {
    dont_take_focus: bool,
    jump_back: bool
}

impl Flags {
    fn new() -> Self {
        let mut args = std::env::args().into_iter();
        args.next();
        
        let mut dont_take_focus = false;
        let mut jump_back = false;

        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--dont-take-focus" => {
                    dont_take_focus = true;
                }
                "--jump-back" => {
                    jump_back = true;
                }
                s => {
                    println!("option {s} not recognized");
                }
            }
        }
        
        Self {
            dont_take_focus,
            jump_back
        }
    }
}

fn main() {

    let package = process();
    let cwd_current_tab = package.windows[package.i_current_window].cwd.clone();
    let id_window_current = package.windows[package.i_current_window].id;
    // dbg!(&package.windows);

    let flags = Flags::new();
    
    let id_window_runner: isize = if let Some(id_window) = choose(package.i_current_window, package.windows) {
        // println!("dont_take_focusing window {id_window_runner}");
        if !flags.dont_take_focus {
            focus_window(id_window);
        }
        id_window as isize
    } else {
        // println!("launching new tab");
        // select window
        new_tab(&cwd_current_tab, flags.dont_take_focus);
        -1 as isize
    };
    cargo(id_window_runner);
    if flags.jump_back {
        focus_window(id_window_current);
    }
}
