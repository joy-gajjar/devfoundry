# Native Integration Harnesses

The `devfoundry-tools` integration test target exercises local child-process
boundaries without external servers, provider credentials, network access, or
repository secrets.

Run it with:

```text
cargo test -p devfoundry-tools --test native_integrations
```

The repository-owned `local-integration-fixture` binary provides a minimal
Content-Length JSON-RPC MCP server and a quiet long-lived LSP child. MCP is
compiled and tested only on macOS because the production adapter is macOS-only.
LSP and PTY tests use temporary directories, bounded waits, and the existing
permission broker. PTY currently verifies the pipe-backed terminal contract,
not native terminal allocation or restoration.

No test reads credentials or contacts an external service. Permission-denial
tests assert that the distinct integration operation is checked before work.
