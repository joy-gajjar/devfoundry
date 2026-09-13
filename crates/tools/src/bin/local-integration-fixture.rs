use std::io::{self, Read, Write};

fn main() -> io::Result<()> {
    match std::env::args().nth(1).as_deref() {
        Some("mcp") => mcp_fixture(),
        Some("lsp") => lsp_fixture(),
        _ => Ok(()),
    }
}

fn mcp_fixture() -> io::Result<()> {
    let mut input = io::stdin().lock();
    let mut output = io::stdout().lock();
    loop {
        let Some(body) = read_line_frame(&mut input)? else {
            return Ok(());
        };
        let request: serde_json::Value = serde_json::from_slice(&body).map_err(io::Error::other)?;
        let Some(id) = request.get("id").cloned() else {
            continue;
        };
        let result = match request.get("method").and_then(serde_json::Value::as_str) {
            Some("initialize") => {
                serde_json::json!({"protocolVersion": "2024-11-05", "capabilities": {"tools": {}}})
            }
            Some("tools/list") => {
                serde_json::json!({"tools": [{"name": "local", "description": "fixture", "inputSchema": {"type": "object"}}]})
            }
            Some("tools/call") => serde_json::json!({
                "content": [{"type": "text", "text": "local fixture"}],
                "isError": false
            }),
            _ => serde_json::json!({}),
        };
        write_line_frame(
            &mut output,
            &serde_json::json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": result
            }),
        )?;
    }
}

fn lsp_fixture() -> io::Result<()> {
    let mut input = io::stdin().lock();
    let mut output = io::stdout().lock();
    loop {
        let Some(body) = read_lsp_frame(&mut input)? else {
            return Ok(());
        };
        let request: serde_json::Value = serde_json::from_slice(&body).map_err(io::Error::other)?;
        let Some(id) = request.get("id").cloned() else {
            continue;
        };
        match request.get("method").and_then(serde_json::Value::as_str) {
            Some("initialize") => write_lsp_frame(
                &mut output,
                &serde_json::json!({
                    "jsonrpc": "2.0", "id": id, "result": {"capabilities": {"diagnosticProvider": {}}}
                }),
            )?,
            Some("shutdown") => write_lsp_frame(
                &mut output,
                &serde_json::json!({
                    "jsonrpc": "2.0", "id": id, "result": null
                }),
            )?,
            Some("textDocument/diagnostic") => {
                write_lsp_frame(
                    &mut output,
                    &serde_json::json!({
                        "jsonrpc": "2.0", "method": "textDocument/publishDiagnostics",
                        "params": {"uri": "file:///main.rs", "version": 1,
                        "diagnostics": [{"range": {"start": {"line": 0, "character": 0}, "end": {"line": 0, "character": 1}}, "severity": 2, "message": "local fixture warning"}]}
                    }),
                )?;
                write_lsp_frame(
                    &mut output,
                    &serde_json::json!({
                        "jsonrpc": "2.0", "id": id, "result": {"kind": "full", "items": []}
                    }),
                )?;
            }
            _ => write_lsp_frame(
                &mut output,
                &serde_json::json!({
                    "jsonrpc": "2.0", "id": id, "result": null
                }),
            )?,
        }
    }
}

fn read_frame(reader: &mut impl Read) -> io::Result<Option<Vec<u8>>> {
    let mut headers = Vec::new();
    let mut byte = [0_u8; 1];
    loop {
        if reader.read(&mut byte)? == 0 {
            return Ok(None);
        }
        headers.push(byte[0]);
        if headers.ends_with(b"\r\n\r\n") {
            break;
        }
        if headers.len() > 8 * 1024 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "headers too large",
            ));
        }
    }
    let headers = String::from_utf8(headers).map_err(io::Error::other)?;
    let length = headers
        .lines()
        .find_map(|line| line.strip_prefix("Content-Length:")?.trim().parse().ok())
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "missing length"))?;
    if length > 256 * 1024 {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "body too large"));
    }
    let mut body = vec![0; length];
    reader.read_exact(&mut body)?;
    Ok(Some(body))
}

fn read_line_frame(reader: &mut impl Read) -> io::Result<Option<Vec<u8>>> {
    let mut body = Vec::new();
    let mut byte = [0_u8; 1];
    loop {
        if reader.read(&mut byte)? == 0 {
            return Ok(None);
        }
        if byte[0] == b'\n' {
            break;
        }
        body.push(byte[0]);
        if body.len() > 256 * 1024 {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "line too large"));
        }
    }
    if body.starts_with(b"Content-Length:") {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Content-Length rejected",
        ));
    }
    Ok(Some(body))
}

fn write_line_frame(writer: &mut impl Write, value: &serde_json::Value) -> io::Result<()> {
    let body = serde_json::to_vec(value).map_err(io::Error::other)?;
    writer.write_all(&body)?;
    writer.write_all(b"\n")?;
    writer.flush()
}

fn read_lsp_frame(reader: &mut impl Read) -> io::Result<Option<Vec<u8>>> {
    read_frame(reader)
}

fn write_lsp_frame(writer: &mut impl Write, value: &serde_json::Value) -> io::Result<()> {
    let body = serde_json::to_vec(value).map_err(io::Error::other)?;
    write!(writer, "Content-Length: {}\r\n\r\n", body.len())?;
    writer.write_all(&body)?;
    writer.flush()
}
