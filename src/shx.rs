pub(crate) fn is_macos_command() -> &'static str {
    r#"test "$(uname -s)" = Darwin"#
}

pub(crate) fn euid_command() -> &'static str {
    r#"id -u"#
}

pub(crate) fn file_owner_command(file_expr_raw: &str) -> String {
    format!(r#"stat -f %u {}"#, file_expr_raw)
}

pub(crate) fn exec_if_exists(exe_path: &str) -> String {
    format!(r#"test -x "{}" && exec "{}" "$@""#, exe_path, exe_path)
}
