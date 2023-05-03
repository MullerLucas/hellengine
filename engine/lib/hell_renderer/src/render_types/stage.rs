use hell_core::error::{HellError, OptToHellErr};

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[repr(usize)]
pub enum ShaderStageType {
    Vertex = 0,
    Fragment
}

impl TryFrom<&str> for ShaderStageType {
    type Error = HellError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        ShaderStageType::parse_txt(value).ok_or_render_herr("failed to parse value into shader type")
    }
}

impl ShaderStageType {
    pub const SHADER_TYPE_COUNT: usize = 2;

    fn parse_txt(txt: &str) -> Option<Self> {
        match txt.trim_start() {
            "vert"|"VERT"|"Vert"|"vertex"|"VERTEX"|"Vertex"       => Some(Self::Vertex),
            "frag"|"FRAG"|"Frag"|"fragment"|"FRAGMENT"|"Fragment" => Some(Self::Fragment),
            _ => None,
        }
    }
}
