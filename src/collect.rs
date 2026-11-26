use std::{
    path::{Path, PathBuf},
    str::FromStr,
};

use serde::Deserialize;
use strum::EnumString;
use thiserror::Error;
use walkdir::WalkDir;

#[derive(Debug, PartialEq, EnumString, Clone)]
pub enum Specialisation {
    #[strum(ascii_case_insensitive, serialize = "dev", serialize = "development")]
    Development,

    #[strum(ascii_case_insensitive, serialize = "prod", serialize = "production")]
    Production,
}

#[derive(Debug, PartialEq, Clone)]
pub struct FileInfo {
    pub service_name: String,
    pub path: PathBuf,
    pub tag: Option<String>,
    pub specialisation: Option<Specialisation>,
}

#[derive(Debug, PartialEq, Eq)]
struct FileName {
    service_name: String,
    tag: Option<String>,
}

impl FromStr for FileName {
    type Err = ();

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        // Look for "compose.$SERVICE[.$TAG].yaml"
        let mut parts = s.split(".");

        // The file name must be prefixed with "compose"
        if parts.next().ok_or(())? != "compose" {
            return Err(());
        }

        // The next part is the service name
        let service_name = parts.next().ok_or(())?.to_string();

        // This might be the optional tag or the trailing "yaml"
        let maybe_tag = parts.next().ok_or(())?;

        let tag = if maybe_tag == "yaml" {
            None
        } else {
            // In thise case, maybe_tag is the real tag
            // Check for the trailing yaml
            if parts.next().ok_or(())? != "yaml" {
                return Err(());
            }

            Some(maybe_tag.to_string())
        };

        // Check that nothing else is left
        if parts.next().is_some() {
            return Err(());
        }

        Ok(Self { service_name, tag })
    }
}

#[derive(Debug, Error)]
pub enum CollectError {
    #[error("io error")]
    Io(#[from] std::io::Error),

    #[error("strum parse error")]
    InvalidEnum(#[from] strum::ParseError),

    #[error("serde parse error")]
    Serde(#[from] serde_saphyr::DeserializeError),

    #[error("walkdir error")]
    WalkDir(#[from] walkdir::Error),
}

pub fn collect_files(root_dir: &str) -> Result<Vec<FileInfo>, CollectError> {
    let mut info = Vec::new();

    for entry in WalkDir::new(root_dir) {
        let entry = entry?;

        let file_name = match FileName::from_str(&entry.file_name().to_string_lossy()) {
            Ok(f) => f,
            Err(_) => continue,
        };

        let specialisation: Option<_> = if let Some(tag) = file_name.tag.as_deref() {
            // If the tag is a valid spec, use that. Otherwise, read the file to look for a spec.
            if let Ok(spec) = Specialisation::from_str(tag) {
                Some(spec)
            } else {
                get_spec_from_file(entry.path())?
            }
        } else {
            // If there is no tag, read the file to check for a spec.
            get_spec_from_file(entry.path())?
        };

        info.push(FileInfo {
            service_name: file_name.service_name,
            path: entry.path().to_path_buf(),
            tag: file_name.tag,
            specialisation,
        });
    }

    Ok(info)
}

/// Given a path, read the file and extract the specialisation using the `x-compose-builder-spec`
/// key. If no specialisation exists, return Ok(None).
fn get_spec_from_file<P: AsRef<Path>>(path: P) -> Result<Option<Specialisation>, CollectError> {
    let contents = std::fs::read_to_string(path)?;
    get_spec_from_str(&contents)
}

/// Given a YAML string, extract the specialisation using the `x-compose-builder-spec` key. If no
/// specialisation exists, return Ok(None).
fn get_spec_from_str(text: &str) -> Result<Option<Specialisation>, CollectError> {
    #[derive(Deserialize)]
    struct StrSpec<'a> {
        #[serde(rename = "x-compose-builder-spec")]
        spec: Option<&'a str>,
    }

    // Check if `x-compose-builder-spec` is present. If it isn't, assume no spec. If it is, require
    // it to be a valid spec.
    let str_spec: StrSpec = serde_saphyr::from_str(text)?;

    // If a spec was read, make sure it's valid. If it wasn't, assume no spec.
    Ok(if let Some(s) = str_spec.spec {
        Some(Specialisation::from_str(s)?)
    } else {
        None
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_name_parse() {
        let file_name =
            FileName::from_str("compose.foo.yaml").expect("unable to parse valid filename");
        assert_eq!(
            file_name,
            FileName {
                service_name: "foo".to_string(),
                tag: None,
            },
        );

        let file_name =
            FileName::from_str("compose.foo.bar.yaml").expect("unable to parse valid filename");
        assert_eq!(
            file_name,
            FileName {
                service_name: "foo".to_string(),
                tag: Some("bar".to_string()),
            },
        );
    }

    #[test]
    fn test_file_name_parse_invalid() {
        FileName::from_str("compose.yaml").expect_err("parsed file with no service name");

        FileName::from_str("compose.foo.yam").expect_err("parsed file with invalid extension");

        FileName::from_str("compos.foo.yaml").expect_err("parsed file with invalid prefix");

        FileName::from_str("foo.yaml").expect_err("parsed file with no prefix");

        FileName::from_str("compose.foo").expect_err("parsed file with no extension");

        FileName::from_str("compose.foo.bar.baz.yaml").expect_err("parsed file with an extra tag");
    }

    #[test]
    fn test_specialisation_from_str() {
        Specialisation::from_str("dev").unwrap();
        Specialisation::from_str("development").unwrap();
        Specialisation::from_str("Dev").unwrap();
        Specialisation::from_str("Development").unwrap();
        Specialisation::from_str("DEV").unwrap();
        Specialisation::from_str("DEVELOPMENT").unwrap();

        Specialisation::from_str("prod").unwrap();
        Specialisation::from_str("production").unwrap();
        Specialisation::from_str("Prod").unwrap();
        Specialisation::from_str("Production").unwrap();
        Specialisation::from_str("PROD").unwrap();
        Specialisation::from_str("PRODUCTION").unwrap();
    }

    #[test]
    fn test_get_spec_from_str() {
        // Valid cases
        let yaml_str = "";
        let spec = get_spec_from_str(yaml_str).expect("unable to parse valid yaml");
        assert!(spec.is_none(), "spec was returned for no spec");

        let yaml_str = "x-compose-builder-spec: dev";
        let spec = get_spec_from_str(yaml_str)
            .expect("unable to parse valid yaml")
            .expect("no spec returned for dev spec");
        assert_eq!(spec, Specialisation::Development);

        let yaml_str = "x-compose-builder-spec: prod";
        let spec = get_spec_from_str(yaml_str)
            .expect("unable to parse valid yaml")
            .expect("no spec returned for prod spec");
        assert_eq!(spec, Specialisation::Production);

        let yaml_str = "x-compose-builder-spec: development";
        let spec = get_spec_from_str(yaml_str)
            .expect("unable to parse valid yaml")
            .expect("no spec returned for dev spec");
        assert_eq!(spec, Specialisation::Development);

        let yaml_str = "x-compose-builder-spec: production";
        let spec = get_spec_from_str(yaml_str)
            .expect("unable to parse valid yaml")
            .expect("no spec returned for prod spec");
        assert_eq!(spec, Specialisation::Production);

        let yaml_str = r"x-compose-builder-spec: prod

services:
  foo:
    environment:
      NUMBER=42
";
        let spec = get_spec_from_str(yaml_str)
            .expect("unable to parse valid yaml")
            .expect("no spec returned for prod spec");
        assert_eq!(spec, Specialisation::Production);

        // Invalid cases
        let yaml_str = r#"x-compose-builder-spec: "dev"#;
        let err = get_spec_from_str(yaml_str).expect_err("parsed invalid yaml");
        assert!(
            matches!(err, CollectError::Serde(_)),
            "wrong error returned"
        );

        let yaml_str = r"x-compose-builder-spec: staging";
        let err = get_spec_from_str(yaml_str).expect_err("parsed invalid spec");
        assert!(
            matches!(err, CollectError::InvalidEnum(_)),
            "wrong error returned"
        );
    }
}
