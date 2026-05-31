pub type Selection = u32;
pub type SelectionFlag = u32;

pub trait SelectionTrait{
    fn new() -> Self;
    /// Returns true if selection contains a flag
    fn test(&self, flag: SelectionFlag) -> bool;
    /// Adds flag to selection
    fn set(&mut self, flag: SelectionFlag);
    /// Clears flag from selection
    fn clear(&mut self, flag: SelectionFlag);
}

impl SelectionTrait for Selection{
    fn new() -> Self {
        return 0;
    }
    fn test(&self, flag: SelectionFlag) -> bool {
        return (self & flag) != 0
    }
    fn set(&mut self, flag: SelectionFlag) {
        *self = *self | flag
    }
    fn clear(&mut self, flag: SelectionFlag) {
        *self = *self & (!flag)
    }
}
// Flags
pub const API_URL: SelectionFlag = 0b1;
pub const API_KEY: SelectionFlag = 0b10;

#[test]
fn selection(){
    let mut selection = Selection::new();
    selection.set(API_URL);
    selection.set(API_KEY);
    assert_eq!(selection.test(API_URL), true);
    assert_eq!(selection.test(API_KEY), true);

    selection.clear(API_URL);
    assert_eq!(selection.test(API_URL), false);
    assert_eq!(selection.test(API_KEY), true);

    selection.set(API_URL);
    assert_eq!(selection.test(API_URL), true);
    assert_eq!(selection.test(API_KEY), true);

    selection.clear(API_KEY);
    assert_eq!(selection.test(API_URL), true);
    assert_eq!(selection.test(API_KEY), false);
}