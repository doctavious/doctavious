pub struct CleanUp<'a> {
    closure: &'a dyn Fn() -> (),
}

impl CleanUp<'_> {
    pub fn new(closure: &dyn Fn() -> ()) -> Self {
        Self { closure }
    }
}

impl Drop for CleanUp<'_> {
    fn drop(&mut self) {
        let _ = &(self.closure)();
    }
}
