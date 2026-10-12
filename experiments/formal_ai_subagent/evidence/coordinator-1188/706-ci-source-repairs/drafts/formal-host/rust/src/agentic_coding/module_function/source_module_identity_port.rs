//! Host module identity resolution; no authoring or runtime-effect authority.
pub trait SourceModuleIdentityHost {
    fn module_url(&self, specifier: &str, parent: &str) -> Result<String, &'static str>;
}
pub fn source_operation_module_url<Host: SourceModuleIdentityHost>(
    host: Option<&Host>,
    specifier: &str,
    parent: &str,
) -> Result<String, &'static str> {
    host.ok_or("MissingSourceModuleIdentityHost")?
        .module_url(specifier, parent)
}
