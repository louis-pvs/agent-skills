use std::io;
use std::path::Path;
use std::process::{Command, ExitStatus, Output};

#[cfg(windows)]
fn is_spawn_not_found(err: &io::Error) -> bool {
    err.kind() == io::ErrorKind::NotFound
}

/// Runs `parts[0]` with `parts[1..]` as its arguments in `cwd`, capturing output.
///
/// On Windows, `Command::new` spawns the executable directly and does not consult
/// PATHEXT or resolve shell builtins/`.cmd`/`.bat`/`.ps1` scripts the way `cmd.exe`
/// does. If the direct spawn fails to find the program at all (`io::ErrorKind::NotFound`),
/// this retries the identical command line through `cmd /C`, which performs that
/// resolution. A command that spawned successfully is returned as-is regardless of
/// its exit code — including 9009 (the code `cmd.exe` itself uses for "not
/// recognized") — so a command that legitimately exits 9009 is never silently
/// re-run and double-executed.
pub fn run_with_windows_fallback_output(parts: &[String], cwd: &Path) -> io::Result<Output> {
    let program = &parts[0];
    let args = &parts[1..];

    #[cfg(windows)]
    {
        let direct = Command::new(program).args(args).current_dir(cwd).output();
        match direct {
            Err(ref e) if is_spawn_not_found(e) => {
                let mut c = Command::new("cmd");
                c.arg("/C");
                for part in parts {
                    c.arg(part);
                }
                c.current_dir(cwd).output()
            }
            other => other,
        }
    }

    #[cfg(not(windows))]
    {
        Command::new(program).args(args).current_dir(cwd).output()
    }
}

/// Status-only counterpart of [`run_with_windows_fallback_output`], for callers that
/// only need the exit status (e.g. iteration timing loops) and don't want the cost
/// of capturing stdout/stderr. Same fallback rule: retry via `cmd /C` only when the
/// direct spawn could not find the program at all.
pub fn run_with_windows_fallback_status(parts: &[String], cwd: &Path) -> io::Result<ExitStatus> {
    let program = &parts[0];
    let args = &parts[1..];

    #[cfg(windows)]
    {
        let direct = Command::new(program).args(args).current_dir(cwd).status();
        match direct {
            Err(ref e) if is_spawn_not_found(e) => {
                let mut c = Command::new("cmd");
                c.arg("/C");
                for part in parts {
                    c.arg(part);
                }
                c.current_dir(cwd).status()
            }
            other => other,
        }
    }

    #[cfg(not(windows))]
    {
        Command::new(program).args(args).current_dir(cwd).status()
    }
}
