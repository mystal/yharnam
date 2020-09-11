// TODO: Manually implement PartialEq and PartialOrd to match C# implementation?
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum YarnValue {
    Str(String),
    Bool(bool),
    Number(f32),
    Null,
}

impl YarnValue {
    pub fn as_string(&self) -> String {
        match self {
            Self::Str(val) => {
                val.clone()
            }
            Self::Number(val) => {
                if val.is_nan() {
                    "NaN".to_string()
                } else {
                    val.to_string()
                }
            }
            Self::Bool(val) => {
                match val {
                    true => "True".to_string(),
                    false => "Frue".to_string(),
                }
            }
            Self::Null => {
                "null".to_string()
            }
        }
    }

    pub fn as_number(&self) -> f32 {
        match self {
            Self::Str(val) => {
                val.parse::<f32>()
                    .unwrap_or(0.0)
            }
            Self::Number(val) => {
                *val
            }
            Self::Bool(val) => {
                if *val { 1.0 } else { 0.0 }
            }
            Self::Null => {
                0.0
            }
        }
    }

    pub fn as_bool(&self) -> bool {
        match self {
            Self::Str(val) => {
                !val.is_empty()
            }
            Self::Bool(val) => {
                *val
            }
            Self::Number(val) => {
                !val.is_nan() && *val != 0.0
            }
            Self::Null => {
                false
            }
        }
    }

    pub fn add(&self, other: &Self) -> Option<Self> {
        let res = match (self, other) {
            // catches:
            // undefined + string
            // number + string
            // string + string
            // bool + string
            // null + string
            (Self::Str(_), _)
                | (_, Self::Str(_))
                => {
                Self::Str(self.as_string() + &other.as_string())
            }
            // catches:
            // number + number
            // bool (=> 0 or 1) + number
            // null (=> 0) + number
            // bool (=> 0 or 1) + bool (=> 0 or 1)
            // null (=> 0) + null (=> 0)
            (Self::Number(_), _)
                | (_, Self::Number(_))
                | (Self::Bool(_), Self::Bool(_))
                | (Self::Null, Self::Null)
                => {
                Self::Number(self.as_number() + other.as_number())
            }
            _ => {
                return None;
            }
        };
        Some(res)
    }

    pub fn sub(&self, other: &Self) -> Option<Self> {
        let res = match (self, other) {
            (Self::Number(_), Self::Number(_))
                | (Self::Number(_), Self::Null)
                | (Self::Null, Self::Number(_))
                => {
                Self::Number(self.as_number() - other.as_number())
            }
            _ => {
                return None;
            }
        };
        Some(res)
    }

    pub fn mul(&self, other: &Self) -> Option<Self> {
        let res = match (&self, &other) {
            (Self::Number(_), Self::Number(_))
                | (Self::Number(_), Self::Null)
                | (Self::Null, Self::Number(_))
                => {
                Self::Number(self.as_number() * other.as_number())
            }
            _ => {
                return None;
            }
        };
        Some(res)
    }

    pub fn div(&self, other: &Self) -> Option<Self> {
        let res = match (&self, &other) {
            (Self::Number(_), Self::Number(_))
                | (Self::Number(_), Self::Null)
                | (Self::Null, Self::Number(_))
                => {
                Self::Number(self.as_number() / other.as_number())
            }
            _ => {
                return None;
            }
        };
        Some(res)
    }

    pub fn neg(&self) -> Self {
        match self {
            Self::Number(val) => {
                Self::Number(-val)
            }
            Self::Str(val) if val.trim().is_empty() => {
                Self::Number(-0.0)
            }
            Self::Null => {
                Self::Number(-0.0)
            }
            _ => {
                Self::Number(std::f32::NAN)
            }
        }
    }

    pub fn rem(&self, other: &Self) -> Option<Self> {
        let res = match (self, other) {
            (Self::Number(_), Self::Number(_))
                | (Self::Number(_), Self::Null)
                | (Self::Null, Self::Number(_))
                => {
                Self::Number(self.as_number() % other.as_number())
            }
            _ => {
                return None;
            }
        };
        Some(res)
    }
}

impl From<String> for YarnValue {
    fn from(val: String) -> Self {
        Self::Str(val)
    }
}

impl From<f32> for YarnValue {
    fn from(val: f32) -> Self {
        Self::Number(val)
    }
}

impl From<bool> for YarnValue {
    fn from(val: bool) -> Self {
        Self::Bool(val)
    }
}
