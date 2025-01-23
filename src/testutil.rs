#![cfg(test)]

use std::process::{Command, Stdio};

pub(crate) fn assert_shell_ok(command: impl AsRef<str>) {
    let command = command.as_ref();
    for shell in ["sh", "bash"] {
        let status = Command::new(shell).arg("-c").arg(command)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .stdin(Stdio::null())
            .status().unwrap();
        assert!(status.success(), "shell {shell}, command {command} failed");
    }
}

pub(crate) fn assert_shell_err(command: impl AsRef<str>) {
    let command = command.as_ref();
    for shell in ["sh", "bash"] {
        let status = Command::new(shell).arg("-c").arg(command)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .stdin(Stdio::null())
        .status().unwrap();
        assert!(!status.success(), "shell {shell}, command {command} succeeded");
    }
}
