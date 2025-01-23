use crate::os::Os;
use crate::sh::{ShArg, ShArgEscape, ShArgRaw};
use crate::shx;
use crate::shx::{euid_command, exec_if_exists, file_owner_command};
use crate::spec::resolve::{ResolvedSpec, ResolvedTargetSpec};
use crate::target_platform::TargetPlatform;
use std::fmt::{Display, Write};

#[must_use]
fn die_impl(message: impl Into<ShArg>) -> String {
    format!(r#"echo {} >&2; exit 1"#, message.into())
}

#[must_use]
fn die(message: impl Display) -> String {
    die_impl(ShArgEscape(message.to_string()))
}

fn commaslash_dir(os: &Os) -> String {
    format!("{}/commaslash", os.cache_dir_expr())
}

fn backup_commaslash_dir() -> &'static str {
    "${TMPDIR:-/tmp}/commaslash-$(id -u)"
}

fn install_dir(target_platform: &TargetPlatform, spec: &ResolvedTargetSpec) -> String {
    format!("{}/{}", commaslash_dir(&target_platform.os()), spec.sha256)
}

fn backup_install_dir(spec: &ResolvedTargetSpec) -> String {
    format!("{}/{}", backup_commaslash_dir(), spec.sha256)
}

fn exe_path(target_platform: &TargetPlatform, spec: &ResolvedTargetSpec) -> String {
    format!("{}/{}", install_dir(target_platform, spec), spec.path)
}

fn backup_exe_path(spec: &ResolvedTargetSpec) -> String {
    format!("{}/{}", backup_install_dir(spec), spec.path)
}

pub(crate) struct Gen {
    script: String,
    indent: usize,
}

impl Write for Gen {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        for c in s.chars() {
            if c == '\n' {
                self.script.push(c);
            } else {
                if self.script.ends_with('\n') {
                    self.script
                        .push_str(&format!("{:width$}", "", width = self.indent * 4));
                }
                self.script.push(c);
            }
        }
        Ok(())
    }
}

impl Gen {
    pub(crate) fn indented(
        &mut self,
        f: impl FnOnce(&mut Self) -> anyhow::Result<()>,
    ) -> anyhow::Result<()> {
        self.indent += 1;
        f(self)?;
        self.indent -= 1;
        Ok(())
    }

    fn case(
        &mut self,
        expr: ShArgRaw,
        body: impl FnOnce(&mut Self) -> anyhow::Result<()>,
    ) -> anyhow::Result<()> {
        writeln!(self, r#"case {expr} in"#)?;
        self.indented(body)?;
        writeln!(self, "esac")?;
        Ok(())
    }

    fn if_fi(
        &mut self,
        expr: &str,
        body: impl FnOnce(&mut Self) -> anyhow::Result<()>,
    ) -> anyhow::Result<()> {
        writeln!(self, "if {expr}; then")?;
        self.indented(body)?;
        writeln!(self, "fi")?;
        Ok(())
    }

    fn fast_path(&mut self, spec: &ResolvedSpec) -> anyhow::Result<()> {
        self.case(ShArgRaw(r#""$(uname -sm)""#.to_owned()), |gen| {
            for (target_platform, spec) in &spec.specs {
                let exe_path = exe_path(&target_platform, spec);
                let backup_exe_path = backup_exe_path(spec);
                writeln!(gen, r#""{}")"#, target_platform.uname_sm(),)?;
                for path in &[exe_path, backup_exe_path] {
                    writeln!(gen, r#"    {}"#, exec_if_exists(path))?;
                }
                writeln!(gen, r#"    ;;"#,)?;
            }
            writeln!(
                gen,
                r#"*) {} ;;"#,
                die_impl(ShArgRaw(r#""unsupported pair: $(uname -sm)""#.to_owned()))
            )?;
            Ok(())
        })?;
        Ok(())
    }

    fn _define_die(&mut self) -> anyhow::Result<()> {
        writeln!(self, r#"die() {{"#)?;
        writeln!(self, r#"    echo "$@" >&2"#)?;
        writeln!(self, r#"    exit 1"#)?;
        writeln!(self, r#"}}"#)?;
        Ok(())
    }

    fn die(&mut self, message: impl Display) -> anyhow::Result<()> {
        writeln!(self, "{}", die(message))?;
        Ok(())
    }

    fn assert_command_exists(&mut self, command: &str, test_command: &str) -> anyhow::Result<()> {
        self.if_fi(&format!("! {test_command} >/dev/null 2>&1"), |gen| {
            gen.die(format_args!("command `{command}` not found"))?;
            Ok(())
        })
    }

    fn subprocess(
        &mut self,
        cb: impl FnOnce(&mut Self) -> anyhow::Result<()>,
    ) -> anyhow::Result<()> {
        writeln!(self, "(")?;
        self.indented(cb)?;
        writeln!(self, ")")?;
        Ok(())
    }

    fn comment(&mut self, comment: impl Display) -> anyhow::Result<()> {
        writeln!(self, "# {}", comment)?;
        Ok(())
    }

    fn assign_raw(&mut self, var: &str, value: impl Display) -> anyhow::Result<()> {
        writeln!(self, "{var}={value}")?;
        Ok(())
    }

    fn slow_path_for_target(
        &mut self,
        target_platform: &TargetPlatform,
        spec: &ResolvedTargetSpec,
    ) -> anyhow::Result<()> {
        let sha256_command = target_platform.os().sha256_command();

        self.assert_command_exists(
            spec.archive_format.command(),
            spec.archive_format.test_command(),
        )?;
        self.assert_command_exists(
            sha256_command.command(),
            sha256_command.test_command().as_str(),
        )?;

        writeln!(
            self,
            r#"commaslash_dir="{}""#,
            commaslash_dir(&target_platform.os())
        )?;

        self.comment("Checking if we own the directory")?;
        writeln!(self, r#"_commaslash_dir_x="$commaslash_dir""#)?;
        writeln!(self, r#"while ! test -e "$_commaslash_dir_x"; do"#)?;
        writeln!(
            self,
            r#"    _commaslash_dir_x="$(dirname "$_commaslash_dir_x")""#
        )?;
        writeln!(self, r#"done"#)?;
        self.comment("If we don't own the directory, we will cache in a temporary directory")?;
        writeln!(
            self,
            r#"if test "$({})" -ne "$({})"; then"#,
            file_owner_command(r#""$_commaslash_dir_x""#),
            euid_command()
        )?;
        writeln!(
            self,
            r#"    commaslash_dir="${{TMPDIR:-/tmp}}/commaslash-$({})""#,
            euid_command()
        )?;
        writeln!(self, r#"fi"#)?;

        writeln!(self, r#"install_dir="$commaslash_dir/{}""#, spec.sha256)?;
        writeln!(self, r#"temp_dir="$install_dir.temp""#)?;
        // Temp dir may be left after another run (although this is very improbable).
        writeln!(self, r#"rm -rf "$temp_dir""#)?;
        writeln!(self, r#"mkdir -p "$temp_dir""#)?;
        writeln!(self, r#"trap "rm -rf $temp_dir" EXIT"#)?;
        writeln!(self, r#"mkdir -p "$(dirname "$install_dir")""#)?;
        writeln!(self, r#"exe="$install_dir/{}""#, spec.path)?;
        self.subprocess(|gen| {
            writeln!(gen, r#"exec 9>"$install_dir.lock""#)?;
            gen.comment("It is OK to fail to aquire lock,")?;
            gen.comment("Because we generate unique names and rename atomically.")?;
            gen.comment("If we fail to aquire lock, we will download twice, but we won't corrupt.")?;
            writeln!(gen, "{} || true", target_platform.os().flock().lock_fs(120, 9))?;
            gen.if_fi(r#"test -x "$exe""#, |gen| {
                // Another process has just prepared the directory.
                writeln!(gen, r#"exit 0"#)?;
                Ok(())
            })?;
            writeln!(gen, r#"curl --location --retry 3 --fail --silent --show-error --output "$temp_dir/zip" {}"#,
                shlex::try_quote(&spec.url)?
            )?;
            writeln!(gen, "{} >/dev/null", sha256_command.check_command(spec.sha256, "$temp_dir/zip"))?;
            writeln!(gen, r#"unzip -qq "$temp_dir/zip" -d "$temp_dir/{}" >/dev/null"#, spec.sha256)?;
            writeln!(gen, r#"if ! test -x "$temp_dir/{}/{}"; then"#, spec.sha256, spec.path)?;
            writeln!(gen, r#"   echo "unzipped dir $temp_dir/{} does not have executable file {}" >&2; exit 1"#, spec.sha256, spec.path)?;
            writeln!(gen, r#"fi"#)?;
            writeln!(gen, r#"mv "$temp_dir/{}" "{}""#, spec.sha256, commaslash_dir(&target_platform.os()))?;
            Ok(())
        })?;
        self.comment("We have set up `trap` above, but `trap` is not executed on `exec`")?;
        writeln!(self, r#"rm -rf "$temp_dir""#)?;
        writeln!(self, r#"exec "$exe" "$@""#)?;
        Ok(())
    }

    fn slow_path(&mut self, spec: &ResolvedSpec) -> anyhow::Result<()> {
        writeln!(self, r#"set -e"#)?;
        writeln!(self, r#"case "$(uname -sm)" in"#)?;
        self.indented(|gen| {
            for (target_platform, spec) in &spec.specs {
                writeln!(gen, r#""{}")"#, target_platform.uname_sm())?;
                gen.indented(|gen| {
                    gen.slow_path_for_target(&target_platform, spec)?;
                    writeln!(gen, ";;")?;
                    Ok(())
                })?;
            }
            writeln!(
                gen,
                r#"*) echo "this code should not be reachable; $(uname -sm)" >&2; exit 1 ;;"#
            )?;
            Ok(())
        })?;
        writeln!(self, "esac")?;

        Ok(())
    }

    fn gen(mut self, spec: &ResolvedSpec) -> anyhow::Result<String> {
        writeln!(self, "#!/bin/sh")?;
        writeln!(self, "# {}generated by commaslash", "@")?;

        self.fast_path(spec)?;

        // File not found, let's download it.

        self.slow_path(spec)?;

        Ok(self.script)
    }
}

pub(crate) fn gen(spec: &ResolvedSpec) -> anyhow::Result<String> {
    Gen {
        script: String::new(),
        indent: 0,
    }
    .gen(spec)
}
