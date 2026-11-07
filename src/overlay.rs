#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl Rect {
    #[must_use]
    pub fn valid(self) -> bool {
        self.width > 1 && self.height > 1
    }
}

#[must_use]
pub fn should_show(enabled: bool, focused: bool, bounds: Option<Rect>) -> bool {
    enabled && focused && bounds.is_some_and(Rect::valid)
}

#[cfg(test)]
mod tests {
    use super::{Rect, should_show};

    #[test]
    fn requires_enabled_focused_and_valid_game() {
        let rect = Some(Rect {
            x: -1920,
            y: 0,
            width: 1920,
            height: 1080,
        });
        assert!(should_show(true, true, rect));
        assert!(!should_show(false, true, rect));
        assert!(!should_show(true, false, rect));
        assert!(!should_show(true, true, None));
        assert!(!should_show(
            true,
            true,
            Some(Rect {
                x: 0,
                y: 0,
                width: 1,
                height: 1080
            })
        ));
    }
}
