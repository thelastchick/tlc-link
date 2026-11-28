mod process;

pub use process::{CoreManager, CoreStatus};
use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CoreKind { SingBox, Xray }

impl CoreKind {
    pub fn parse(value: &str) -> Result<Self, String> {
        match value.trim().to_ascii_lowercase().as_str() {
            "sing-box" | "singbox" => Ok(Self::SingBox),
            "xray" => Ok(Self::Xray),
            _ => Err("هسته ناشناخته است؛ فقط sing-box و xray مجازند".into()),
        }
    }
    pub fn name(self) -> &'static str { match self { Self::SingBox => "sing-box", Self::Xray => "xray" } }
    pub fn executable(self) -> &'static str { match self { Self::SingBox => "sing-box.exe", Self::Xray => "xray.exe" } }
    pub fn config_filename(self) -> &'static str { match self { Self::SingBox => "sing-box-runtime.json", Self::Xray => "xray-runtime.json" } }
    pub fn validate_args(self, path: &str) -> Vec<String> {
        match self {
            Self::SingBox => vec!["check".into(), "-c".into(), path.into()],
            Self::Xray => vec!["run".into(), "-test".into(), "-config".into(), path.into()],
        }
    }
    pub fn run_args(self, path: &str) -> Vec<String> {
        match self {
            Self::SingBox => vec!["run".into(), "-c".into(), path.into()],
            Self::Xray => vec!["run".into(), "-config".into(), path.into()],
        }
    }
}
impl fmt::Display for CoreKind { fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { f.write_str(self.name()) } }

#[cfg(test)]
mod tests {
    use super::CoreKind;

    #[test]
    fn core_aliases_are_parsed_case_insensitively() {
        assert!(matches!(CoreKind::parse("SING-BOX"), Ok(CoreKind::SingBox)));
        assert!(matches!(CoreKind::parse("singbox"), Ok(CoreKind::SingBox)));
        assert!(matches!(CoreKind::parse("Xray"), Ok(CoreKind::Xray)));
        assert!(CoreKind::parse("unknown").is_err());
    }

    #[test]
    fn executable_and_config_names_are_platform_specific() {
        assert_eq!(CoreKind::SingBox.executable(), "sing-box.exe");
        assert_eq!(CoreKind::Xray.executable(), "xray.exe");
        assert_eq!(CoreKind::SingBox.config_filename(), "sing-box-runtime.json");
        assert_eq!(CoreKind::Xray.config_filename(), "xray-runtime.json");
    }
}
