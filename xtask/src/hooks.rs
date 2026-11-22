//! `cargo xtask install-hooks [--uninstall]`: an optional pre-push hook that runs the
//! fast checks (`fmt`, `clippy`) before every push.
//!
//! Hooks are never installed automatically. The hook carries a marker line, and xtask
//! only ever overwrites or removes a hook with that marker: a hook someone wrote by
//! hand is left alone, with a warning.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::Exit;
use crate::output;

/// The line that identifies a hook xtask wrote.
const MARKER: &str = "# Installed by `cargo xtask install-hooks`.";

/// The hook's whole content.
const HOOK: &str = "#!/bin/sh
# Installed by `cargo xtask install-hooks`.
# Remove it with `cargo xtask install-hooks --uninstall`.
exec cargo xtask ci --step fmt --step clippy
";

/// What to do with the hook file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Action {
    Write,
    AlreadyInstalled,
    Remove,
    NothingToRemove,
    LeaveForeign,
}

/// Decides from the existing hook, if any, what installing or uninstalling does.
pub(crate) fn plan(existing: Option<&str>, uninstall: bool) -> Action {
    let ours = existing.is_some_and(|text| text.lines().any(|line| line == MARKER));
    match (existing, uninstall) {
        (Some(text), false) if text == HOOK => Action::AlreadyInstalled,
        (None, false) => Action::Write,
        (Some(_), false) if ours => Action::Write,
        (None, true) => Action::NothingToRemove,
        (Some(_), true) if ours => Action::Remove,
        (Some(_), _) => Action::LeaveForeign,
    }
}

pub(crate) fn run(uninstall: bool, out: &mut impl Write) -> Exit {
    let hooks = match hooks_dir() {
        Ok(hooks) => hooks,
        Err(error) => {
            output::line(out, &format!("error: {error}"));
            return Exit::CheckFailed;
        }
    };
    match apply(&hooks.join("pre-push"), uninstall) {
        Ok(message) => {
            output::line(out, &message);
            Exit::Success
        }
        Err(message) => {
            output::line(out, &message);
            Exit::CheckFailed
        }
    }
}

/// Applies the plan to `hook`, returning what happened, or the warning or error.
pub(crate) fn apply(hook: &Path, uninstall: bool) -> Result<String, String> {
    let shown = hook.display();
    let existing = match std::fs::read_to_string(hook) {
        Ok(text) => Some(text),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
        Err(error) => return Err(format!("error: could not read {shown}: {error}")),
    };
    match plan(existing.as_deref(), uninstall) {
        Action::Write => {
            if let Some(dir) = hook.parent() {
                std::fs::create_dir_all(dir).map_err(|error| {
                    format!("error: could not create {}: {error}", dir.display())
                })?;
            }
            std::fs::write(hook, HOOK)
                .map_err(|error| format!("error: could not write {shown}: {error}"))?;
            make_executable(hook)?;
            Ok(format!(
                "Installed {shown}: `git push` now runs `cargo xtask ci --step fmt --step clippy` first."
            ))
        }
        Action::AlreadyInstalled => Ok(format!("{shown} is already installed.")),
        Action::Remove => {
            std::fs::remove_file(hook)
                .map_err(|error| format!("error: could not remove {shown}: {error}"))?;
            Ok(format!("Removed {shown}."))
        }
        Action::NothingToRemove => Ok(format!("No hook at {shown}; nothing to remove.")),
        Action::LeaveForeign => Err(format!(
            "warning: {shown} exists and was not written by xtask; left unchanged. Add \
             `cargo xtask ci --step fmt --step clippy` to it yourself if you want the check."
        )),
    }
}

#[cfg(unix)]
fn make_executable(hook: &Path) -> Result<(), String> {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(hook, std::fs::Permissions::from_mode(0o755)).map_err(|error| {
        format!(
            "error: could not make {} executable: {error}",
            hook.display()
        )
    })
}

#[cfg(not(unix))]
fn make_executable(_hook: &Path) -> Result<(), String> {
    // Git for Windows runs hooks through its own shell, whatever the file mode.
    Ok(())
}

/// The repository's hooks directory, as git resolves it (worktrees and
/// `core.hooksPath` included).
fn hooks_dir() -> Result<PathBuf, String> {
    let root = crate::repo_root();
    let root = root.canonicalize().unwrap_or(root);
    let output = Command::new("git")
        .args(["rev-parse", "--git-path", "hooks"])
        .current_dir(&root)
        .output()
        .map_err(|error| format!("could not run git: {error}"))?;
    if !output.status.success() {
        return Err("`git rev-parse --git-path hooks` failed; is this a git checkout?".to_owned());
    }
    let path = PathBuf::from(String::from_utf8_lossy(&output.stdout).trim());
    Ok(if path.is_absolute() {
        path
    } else {
        root.join(path)
    })
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{Action, HOOK, apply, plan};

    #[test]
    fn the_plan_never_touches_a_foreign_hook() {
        let ours_outdated = "#!/bin/sh\n# Installed by `cargo xtask install-hooks`.\nexec true\n";
        let foreign = "#!/bin/sh\nexec ./my-checks\n";
        assert_eq!(plan(None, false), Action::Write);
        assert_eq!(plan(Some(HOOK), false), Action::AlreadyInstalled);
        assert_eq!(plan(Some(ours_outdated), false), Action::Write);
        assert_eq!(plan(Some(foreign), false), Action::LeaveForeign);
        assert_eq!(plan(None, true), Action::NothingToRemove);
        assert_eq!(plan(Some(HOOK), true), Action::Remove);
        assert_eq!(plan(Some(ours_outdated), true), Action::Remove);
        assert_eq!(plan(Some(foreign), true), Action::LeaveForeign);
    }

    fn temp_hook(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("xtask-hooks-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir); // A leftover from an earlier run, if any.
        dir.join("hooks").join("pre-push")
    }

    #[test]
    fn install_is_idempotent_and_uninstall_removes_it() {
        let hook = temp_hook("idempotent");
        assert!(apply(&hook, false).unwrap().starts_with("Installed"));
        assert_eq!(std::fs::read_to_string(&hook).unwrap(), HOOK);
        assert!(
            apply(&hook, false)
                .unwrap()
                .ends_with("is already installed.")
        );
        assert_eq!(std::fs::read_to_string(&hook).unwrap(), HOOK);
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = std::fs::metadata(&hook).unwrap().permissions().mode();
            assert_eq!(mode & 0o111, 0o111, "{mode:o}");
        }
        assert!(apply(&hook, true).unwrap().starts_with("Removed"));
        assert!(!hook.exists());
        assert!(apply(&hook, true).unwrap().ends_with("nothing to remove."));
    }

    #[test]
    fn a_foreign_hook_is_left_alone_with_a_warning() {
        let hook = temp_hook("foreign");
        std::fs::create_dir_all(hook.parent().unwrap()).unwrap();
        std::fs::write(&hook, "#!/bin/sh\nexec ./my-checks\n").unwrap();
        for uninstall in [false, true] {
            let warning = apply(&hook, uninstall).unwrap_err();
            assert!(warning.starts_with("warning:"), "{warning}");
            assert_eq!(
                std::fs::read_to_string(&hook).unwrap(),
                "#!/bin/sh\nexec ./my-checks\n"
            );
        }
    }
}
