
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
        let last_dot = self.inner_name.rfind('.')?;
        let last_dash = self.inner_name[..last_dot].rfind('-')?;

        let _ = self.inner_name.drain(last_dash..last_dot).count();

        Some(self)
    }
}

#[cfg(test)]
mod test {
    use icon_name::*;

    #[test]
    fn test_icon_name_fallback() {
        let mut icon_name = IconName::from("some-icon-name.svg");
        assert_eq!(icon_name.name(), "some-icon-name.svg");

        icon_name.fallback();
        assert_eq!(icon_name.name(), "some-icon.svg");

        icon_name.fallback();
        assert_eq!(icon_name.name(), "some.svg");

        assert!(icon_name.fallback().is_none());
    }
}