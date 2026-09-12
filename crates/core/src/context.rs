use devfoundry_schema::{ContextManifest, ContextSource};

pub struct ContextManifestBuilder {
    manifest: ContextManifest,
}

impl Default for ContextManifestBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl ContextManifestBuilder {
    pub fn new() -> Self {
        Self {
            manifest: ContextManifest::new(Vec::new()),
        }
    }
    pub fn source(mut self, source: ContextSource) -> Self {
        self.manifest.sources.push(source);
        self
    }
    pub fn unresolved_tool_group(mut self, id: impl Into<String>) -> Self {
        self.manifest.unresolved_tool_groups.push(id.into());
        self
    }
    pub fn build(self) -> ContextManifest {
        self.manifest
    }
}
