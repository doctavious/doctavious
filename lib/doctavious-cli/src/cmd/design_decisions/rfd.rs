use std::path::PathBuf;

// pub(crate) fn init(
//     directory: Option<String>,
//     structure: FileStructure,
//     extension: MarkupFormat,
// ) -> Result<PathBuf> {
//     let mut settings = match load_settings() {
//         Ok(settings) => settings,
//         Err(_) => Default::default(),
//     };
//
//     let dir = match directory {
//         None => DEFAULT_RFD_DIR,
//         Some(ref d) => d,
//     };
//
//     let rfd_settings = RFDSettings {
//         dir: Some(dir.to_string()),
//         structure: Some(structure),
//         template_extension: Some(extension),
//     };
//     settings.rfd_settings = Some(rfd_settings);
//
//     persist_settings(settings)?;
//     init_dir(dir)?;
//
//     // TODO: fix
//     return new_rfd(Some(1), "Use RFDs ...".to_string(), extension);
// }
