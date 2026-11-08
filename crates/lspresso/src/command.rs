use std::{ffi::OsStr, path::PathBuf, process::Command};

/// A command for a prebuilt server. `env_var` overrides `default`, so a test or
/// bench can point at a different build without code changes
pub fn server_command<S: AsRef<OsStr>>(
    env_var: &str,
    default: impl Into<PathBuf>,
    args: impl IntoIterator<Item = S>,
) -> Command {
    let program = std::env::var_os(env_var).map_or_else(|| default.into(), PathBuf::from);
    let mut command = Command::new(program);
    command.args(args);
    command
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn falls_back_to_the_default_when_the_variable_is_unset() {
        let command = server_command("LSPRESSO_SURELY_UNSET", "/bin/server", ["--stdio"]);

        assert_eq!(command.get_program(), "/bin/server");
        assert_eq!(command.get_args().collect::<Vec<_>>(), ["--stdio"]);
    }
}
