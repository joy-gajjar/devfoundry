use devfoundry_core::ContextManifestBuilder;
use devfoundry_schema::ContextSource;

#[test]
fn context_manifest_keeps_current_instructions_and_unresolved_tool_groups() {
    let manifest = ContextManifestBuilder::new()
        .source(ContextSource::new("AGENTS.md", "sha256:1", "instruction"))
        .unresolved_tool_group("call-1")
        .build();
    assert_eq!(manifest.sources.len(), 1);
    assert_eq!(manifest.unresolved_tool_groups, vec!["call-1"]);
}
