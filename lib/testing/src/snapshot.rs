/// From insta patterns https://insta.rs/docs/patterns/
/// Useful for parameterized test frameworks like 'test_case' and 'rtest' to allow associating
/// snapshot with actual test case
#[macro_export]
macro_rules! set_snapshot_suffix {
    ($($expr:expr),*) => {
        let mut settings = insta::Settings::clone_current();
        settings.set_snapshot_suffix(format!($($expr,)*));
        let _guard = settings.bind_to_scope();
    }
}

pub use set_snapshot_suffix;
