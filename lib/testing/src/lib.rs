pub struct CleanUp<F>
where
    F: Fn() -> (),
{
    closure: F,
}

impl<F> CleanUp<F>
where
    F: Fn() -> (),
{
    pub fn new(closure: F) -> Self {
        Self { closure }
    }
}

impl<F> Drop for CleanUp<F>
where
    F: Fn() -> (),
{
    fn drop(&mut self) {
        let _ = &(self.closure)();
    }
}
