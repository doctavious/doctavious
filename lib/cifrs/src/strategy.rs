
pub trait Strategy {

    /// Returns a vec of frameworks associated
    fn execute();

}

pub struct Dependency;

impl Strategy for Dependency {
    fn execute() {
        todo!()
    }
}


pub struct Configuration {
    // content: pattern
}

impl Strategy for Configuration {
    fn execute() {
        todo!()
    }
}

