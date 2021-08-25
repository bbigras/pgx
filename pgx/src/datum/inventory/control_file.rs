use super::{DotIdentifier, SqlGraphEntity, ToSql};
use core::convert::TryFrom;
use std::collections::HashMap;
use tracing_error::SpanTrace;

/// The parsed contents of a `.control` file.
///
/// ```rust
/// use pgx::inventory::ControlFile;
/// use std::convert::TryFrom;
/// # fn main() -> eyre::Result<()> {
/// let context = include_str!("../../../../pgx-examples/custom_types/custom_types.control");
/// let _control_file = ControlFile::try_from(context)?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, Hash, PartialOrd, Ord, PartialEq, Eq)]
pub struct ControlFile {
    pub comment: String,
    pub default_version: String,
    pub module_pathname: String,
    pub relocatable: bool,
    pub superuser: bool,
    pub schema: Option<String>,
}

impl ControlFile {
    /// Parse a `.control` file.
    ///
    /// ```rust
    /// use pgx::inventory::ControlFile;
    /// # fn main() -> eyre::Result<()> {
    /// let context = include_str!("../../../../pgx-examples/custom_types/custom_types.control");
    /// let _control_file = ControlFile::from_str(context)?;
    /// # Ok(())
    /// # }
    /// ```
    #[tracing::instrument(level = "info")]
    pub fn from_str(input: &str) -> Result<Self, ControlFileError> {
        let mut temp = HashMap::new();
        for line in input.lines() {
            let parts: Vec<&str> = line.split('=').collect();

            if parts.len() != 2 {
                continue;
            }

            let (k, v) = (parts.get(0).unwrap().trim(), parts.get(1).unwrap().trim());

            let v = v.trim_start_matches('\'');
            let v = v.trim_end_matches('\'');

            temp.insert(k, v);
        }
        Ok(ControlFile {
            comment: temp
                .get("comment")
                .ok_or(ControlFileError::MissingField {
                    field: "comment",
                    context: SpanTrace::capture(),
                })?
                .to_string(),
            default_version: temp
                .get("default_version")
                .ok_or(ControlFileError::MissingField {
                    field: "default_version",
                    context: SpanTrace::capture(),
                })?
                .to_string(),
            module_pathname: temp
                .get("module_pathname")
                .ok_or(ControlFileError::MissingField {
                    field: "module_pathname",
                    context: SpanTrace::capture(),
                })?
                .to_string(),
            relocatable: temp
                .get("relocatable")
                .ok_or(ControlFileError::MissingField {
                    field: "relocatable",
                    context: SpanTrace::capture(),
                })?
                == &"true",
            superuser: temp
                .get("superuser")
                .ok_or(ControlFileError::MissingField {
                    field: "superuser",
                    context: SpanTrace::capture(),
                })?
                == &"true",
            schema: temp.get("schema").map(|v| v.to_string()),
        })
    }
}

impl Into<SqlGraphEntity> for ControlFile {
    fn into(self) -> SqlGraphEntity {
        SqlGraphEntity::ExtensionRoot(self)
    }
}

/// An error met while parsing a `.control` file.
#[derive(Debug, Clone)]
pub enum ControlFileError {
    MissingField {
        field: &'static str,
        context: SpanTrace,
    },
}

impl std::fmt::Display for ControlFileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ControlFileError::MissingField { field, context } => {
                write!(f, "Missing field in control file! Please add `{}`.", field)?;
                context.fmt(f)?;
            }
        };
        Ok(())
    }
}

impl std::error::Error for ControlFileError {}

impl TryFrom<&str> for ControlFile {
    type Error = ControlFileError;

    fn try_from(input: &str) -> Result<Self, Self::Error> {
        Self::from_str(input)
    }
}

impl ToSql for ControlFile {
    #[tracing::instrument(level = "debug", err, skip(self, _context))]
    fn to_sql(&self, _context: &super::PgxSql) -> eyre::Result<String> {
        let sql = format!(
            "\
            /* \n\
            This file is auto generated by pgx.\n\
            \n\
            The ordering of items is not stable, it is driven by a dependency graph.\n\
            */\
        "
        );
        tracing::debug!(%sql);
        Ok(sql)
    }
}

impl DotIdentifier for ControlFile {
    fn dot_identifier(&self) -> String {
        format!("extension root")
    }
}
