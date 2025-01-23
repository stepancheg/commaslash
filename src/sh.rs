//! This is not correct grammar, but will do for now.

use crate::gen::Gen;
use std::fmt;
use std::fmt::{Display, Formatter, Write};

pub(crate) enum RedirectTarget {
    ToFd(u32),
    ToFile(String),
}
impl Display for RedirectTarget {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            RedirectTarget::ToFd(fd) => write!(f, ">{fd}"),
            RedirectTarget::ToFile(file) => {
                write!(f, ">{}", shlex::try_quote(file).map_err(|_| fmt::Error)?)
            }
        }
    }
}

pub(crate) struct Redirect {
    pub(crate) fd: u32,
    pub(crate) target: RedirectTarget,
}

impl Display for Redirect {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let Redirect { fd, target } = self;
        match fd {
            1 => {}
            fd => write!(f, "{fd}")?,
        }
        write!(f, "{target}")
    }
}

#[derive(derive_more::Display)]
pub(crate) struct ShArgRaw(pub(crate) String);

pub(crate) struct ShArgEscape(pub(crate) String);

impl Display for ShArgEscape {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        shlex::try_quote(&self.0).map_err(|_| fmt::Error)?.fmt(f)
    }
}

#[derive(derive_more::Display, derive_more::From)]
pub(crate) enum ShArg {
    Escape(ShArgEscape),
    Raw(ShArgRaw),
}

pub(crate) struct ShCommand {
    args: Vec<ShArg>,
    redirects: Vec<Redirect>,
}

impl Display for ShCommand {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let ShCommand { args, redirects } = self;
        for (i, arg) in args.iter().enumerate() {
            if i != 0 {
                write!(f, " ")?;
            }
            write!(f, "{}", arg)?;
        }
        for redirect in redirects {
            write!(f, " {redirect}")?;
        }
        Ok(())
    }
}

pub(crate) enum ShCommandOr {
    Command(ShCommand),
    Braced(Box<ShAndOr>),
    CurlyBraced(Box<ShAndOr>),
}

impl Display for ShCommandOr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ShCommandOr::Command(command) => fmt::Display::fmt(command, f),
            ShCommandOr::Braced(command) => {
                write!(f, "( ")?;
                fmt::Display::fmt(command, f)?;
                write!(f, " )")?;
                Ok(())
            }
            ShCommandOr::CurlyBraced(command) => {
                write!(f, "{{ ")?;
                fmt::Display::fmt(command, f)?;
                // Trailing semicolon is needed in bash; not sure about others.
                write!(f, "; }}")?;
                Ok(())
            }
        }
    }
}

#[derive(derive_more::Display)]
pub(crate) enum ShBinOp {
    #[display("&&")]
    And,
    #[display("||")]
    Or,
}

pub(crate) struct ShAndOr {
    first: ShCommandOr,
    rem: Vec<(ShBinOp, ShCommandOr)>,
}

impl Display for ShAndOr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let ShAndOr { first, rem } = self;
        write!(f, "{first}")?;
        for (op, cmd) in rem {
            write!(f, " {op} {cmd}")?;
        }
        Ok(())
    }
}

impl ShAndOr {
    fn writeln(&self, w: &mut Gen) -> anyhow::Result<()> {
        writeln!(w, "{self}")?;
        Ok(())
    }
}

pub(crate) enum ShStmt {
    AndOr(ShAndOr),
    If(ShIf),
}

impl ShStmt {
    fn writeln(&self, w: &mut Gen) -> anyhow::Result<()> {
        match self {
            ShStmt::AndOr(stmt) => stmt.writeln(w),
            ShStmt::If(stmt) => stmt.writeln(w),
        }
    }
}

pub(crate) struct ShVertBlock {
    stmts: Vec<ShStmt>,
}

impl ShVertBlock {
    pub(crate) fn writeln(&self, w: &mut Gen) -> anyhow::Result<()> {
        for stmt in &self.stmts {
            stmt.writeln(w)?;
        }
        Ok(())
    }
}

pub(crate) struct ShIf {
    pub(crate) not: bool,
    pub(crate) cond: ShAndOr,
    pub(crate) body: ShVertBlock,
}

impl ShIf {
    fn writeln(&self, w: &mut Gen) -> anyhow::Result<()> {
        let ShIf { not, cond, body } = self;
        write!(w, "if ")?;
        if *not {
            write!(w, "! ")?;
        }
        write!(w, "{cond}; then")?;
        w.indented(|w| body.writeln(w))?;
        write!(w, "fi")?;
        Ok(())
    }
}
