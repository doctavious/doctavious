use thiserror::Error;

// TODO: need better name than framework
pub enum Framework {
    JavaDoc,
    RustDoc,
    GoDoc,
    JSDoc,
    Maven, // mvn site
    DocFx, // docfx

}