// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use regex::Regex;
use std::env;

use lazy_static::lazy_static;

use crate::prelude::*;

pub const ANKIMATH_VARIANT: &str = "mxyzptlk";

#[derive(Debug, Default)]
pub struct VersionInfo {
    pub name: String,
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    pub prerelease: String,
    pub buildmetadata: String,
    pub buildhash: String,
    pub variant: String,
    pub platform: String,
}

pub fn client_version_re() -> &'static Regex {
    lazy_static! {
        static ref RE: Regex = Regex::new(
            r"(?x)
            ^
            (?:(?P<name>\S+),)?
            (?P<major>0|[1-9]\d*)
            \.
            (?P<minor>0|[1-9]\d*)
            \.
            (?P<patch>0|[1-9]\d*)
            (?:-(?P<prerelease>
                (?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*)
                (?:\.(?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*))*
              )
            )?(?:\+(?P<buildmetadata>
                [0-9a-zA-Z-]+(?:\.[0-9a-zA-Z-]+)*
              )
            )?
            \s+
            \(
            (?P<buildhash>[0-9a-zA-Z-]+)
            \.?
            (?P<variant>[0-9a-zA-Z-]+(?:\.[0-9a-zA-Z-]+)*)?
            \)
            ,
            (?P<platform>\S+)
            $
            "
        )
        .unwrap();
    }
    &RE
}

pub fn short_client_version_re() -> &'static Regex {
    lazy_static! {
        static ref RE: Regex = Regex::new(
            r"(?x)
            ^
            (?P<majorver>0|[1-9]\d*)
            \.
            (?P<minorver>0|[1-9]\d*)
            \.
            (?P<patchver>0|[1-9]\d*)
            (?:-(?P<prerelease>
                (?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*)
                (?:\.(?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*))*
              )
            )?(?:\+(?P<buildmetadata>
                [0-9a-zA-Z-]+(?:\.[0-9a-zA-Z-]+)*
              )
            )?
            ,
            (?P<buildhash>[0-9a-zA-Z-]+)
            \.?
            (?P<variant>[0-9a-zA-Z-]+(?:\.[0-9a-zA-Z-]+)*)?
            ,
            (?P<platform>\S+)
            $
            "
        )
        .unwrap();
    }
    &RE
}

pub fn parse_version_info(version: &str) -> Result<Option<VersionInfo>> {
    println!("parse_version_info(): version: {:?}", version);
    //let caps = client_version_re().captures(version).unwrap();
    match client_version_re().captures(version) {
        Some(caps) => {
            let name: String = match caps.name("name") {
                Some(text) => text.as_str().to_string(),
                _ => "".to_string(),
            };
            let major: u32 = match caps.name("major") {
                Some(text) => text.as_str().parse().unwrap(),
                _ => 0,
            };
            let minor: u32 = match caps.name("minor") {
                Some(text) => text.as_str().parse().unwrap(),
                _ => 0,
            };
            let patch: u32 = match caps.name("patch") {
                Some(text) => text.as_str().parse().unwrap(),
                _ => 0,
            };
            let prerelease: String = match caps.name("prerelease") {
                Some(text) => text.as_str().to_string(),
                _ => "".to_string(),
            };
            let buildmetadata: String = match caps.name("buildmetadata") {
                Some(text) => text.as_str().to_string(),
                _ => "".to_string(),
            };
            let buildhash: String = match caps.name("buildhash") {
                Some(text) => text.as_str().to_string(),
                _ => "".to_string(),
            };
            let variant: String = match caps.name("variant") {
                Some(text) => text.as_str().to_string(),
                _ => "".to_string(),
            };
            let platform: String = match caps.name("platform") {
                Some(text) => text.as_str().to_string(),
                _ => "".to_string(),
            };
            Ok(Some(VersionInfo {
                name,
                major,
                minor,
                patch,
                prerelease,
                buildmetadata,
                buildhash,
                variant,
                platform,
            }))
        }
        None => Ok(None),
    }
}

pub fn parse_short_version_info(version: &str) -> Result<Option<VersionInfo>> {
    println!("parse_short_version_info(): version: {:?}", version);
    //let caps = short_client_version_re().captures(version).unwrap();
    match short_client_version_re().captures(version) {
        Some(caps) => {
            let name: String = "".to_string();
            let major: u32 = match caps.name("major") {
                Some(text) => text.as_str().parse().unwrap(),
                _ => 0,
            };
            let minor: u32 = match caps.name("minor") {
                Some(text) => text.as_str().parse().unwrap(),
                _ => 0,
            };
            let patch: u32 = match caps.name("patch") {
                Some(text) => text.as_str().parse().unwrap(),
                _ => 0,
            };
            let prerelease: String = match caps.name("prerelease") {
                Some(text) => text.as_str().to_string(),
                _ => "".to_string(),
            };
            let buildmetadata: String = match caps.name("buildmetadata") {
                Some(text) => text.as_str().to_string(),
                _ => "".to_string(),
            };
            let buildhash: String = match caps.name("buildhash") {
                Some(text) => text.as_str().to_string(),
                _ => "".to_string(),
            };
            let variant: String = match caps.name("variant") {
                Some(text) => text.as_str().to_string(),
                _ => "".to_string(),
            };
            let platform: String = match caps.name("platform") {
                Some(text) => text.as_str().to_string(),
                _ => "".to_string(),
            };
            Ok(Some(VersionInfo {
                name,
                major,
                minor,
                patch,
                prerelease,
                buildmetadata,
                buildhash,
                variant,
                platform,
            }))
        }
        None => Ok(None),
    }
}

pub fn version() -> &'static str {
    include_str!("../../.version").trim()
}

pub fn buildhash() -> &'static str {
    option_env!("BUILDHASH").unwrap_or("dev").trim()
}

pub(crate) fn sync_client_version() -> &'static str {
    lazy_static! {
        static ref VER: String = format!(
            "anki,{version} ({buildhash}.{variant}),{platform}",
            version = version(),
            buildhash = buildhash(),
            variant = ANKIMATH_VARIANT,
            platform = env::var("PLATFORM").unwrap_or_else(|_| env::consts::OS.to_string())
        );
    }
    &VER
}

pub(crate) fn sync_client_version_short() -> &'static str {
    lazy_static! {
        static ref VER: String = format!(
            "{version},{buildhash}.mxyzptlk,{platform}",
            version = version(),
            buildhash = buildhash(),
            platform = env::consts::OS
        );
    }
    &VER
}

pub(crate) fn sync_server_version() -> &'static str {
    lazy_static! {
        static ref VER: String = format!(
            "Anki/{version}+{variant} ({platform})",
            variant = ANKIMATH_VARIANT,
            version = version(),
            platform = env::consts::OS
        );
    }
    &VER
}
