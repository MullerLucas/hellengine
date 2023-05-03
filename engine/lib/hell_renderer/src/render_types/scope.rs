use hell_core::error::{HellError, HellErrorHelper};

#[derive(Default, Debug, Clone, Copy,  serde::Serialize, serde::Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ShaderScopeType {
    #[default]
    Global = 0,
    Shared,
    Instance,
    Local
}

impl ShaderScopeType {
    pub const SCOPE_COUNT: usize = 4;

    pub fn struct_name(&self) -> &str {
        match self {
            ShaderScopeType::Global   => "GlobalUbo",
            ShaderScopeType::Shared   => "SharedUbo",
            ShaderScopeType::Instance => "InstanceUbo",
            ShaderScopeType::Local    => "LocalUbo"
        }
    }

    pub fn struct_typedef(&self) -> &str {
        match self {
            ShaderScopeType::Global   => "global",
            ShaderScopeType::Shared   => "shared",
            ShaderScopeType::Instance => "instance",
            ShaderScopeType::Local    => "local"
        }
    }
}

impl ShaderScopeType {
    fn parse_txt(txt: &str) -> Option<Self> {
        match txt.trim() {
            "global"|"GLOBAL"|"Global"   => Some(Self::Global),
            "shared"|"SHARED"|"Shared"   => Some(Self::Shared),
            "instance"|"INSTANCE"|"Instance" => Some(Self::Instance),
            "local"|"LOCAL"|"Local"      => Some(Self::Local),
            _ => None,
        }
    }
}

impl TryFrom<&str> for ShaderScopeType {
    type Error = HellError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::parse_txt(value).ok_or_else(|| HellErrorHelper::render_msg_err("failed to parse shader-scope"))
    }
}
