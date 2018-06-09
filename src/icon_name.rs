
use std::convert::From;

bitflags! {
    struct FallbackRules: u32 {
        const SYMBOLIC = 0b00000001;
    }
}

#[derive(Debug, Clone)]
pub struct IconName {
    inner_name: String,
}

impl<T> From<T> for IconName
  where T: AsRef<str> {
    fn from(from: T) -> Self {
        Self {
            inner_name: from.as_ref().to_string(),
        }
    }
}

impl IconName {
    pub fn name(&self) -> &str {
        &self.inner_name
    }

    pub fn fallback(&mut self) -> Option<&IconName> {
        let len = self.inner_name.len();
        let last_dash = self.inner_name.rfind('-')?;

        let _ = self.inner_name.drain(last_dash..len).count();

        Some(self)
    }
}

#[cfg(test)]
mod test {
    use icon_name::*;

    #[test]
    fn test_icon_name_fallback() {
        let mut icon_name = IconName::from("some-icon-name");
        assert_eq!(icon_name.name(), "some-icon-name");

        icon_name.fallback();
        assert_eq!(icon_name.name(), "some-icon");

        icon_name.fallback();
        assert_eq!(icon_name.name(), "some");

        assert!(icon_name.fallback().is_none());
    }
}