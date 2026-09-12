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
        let Some(body) = read_frame(&mut input)? else {
            return Ok(());
        };
        let request: serde_json::Value = serde_json::from_slice(&body).map_err(io::Error::other)?;
        let Some(id) = request.get("id").cloned() else {
            continue;
        };
        let result = match request.get("method").and_then(serde_json::Value::as_str) {
            Some("initialize") => serde_json::json!({"capabilities": {"tools": {}}}),
            Some("tools/call") => serde_json::json!({
                "content": [{"type": "text", "text": "local fixture"}],
                "isError": false
            }),
            _ => serde_json::json!({}),
        };
        write_frame(
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
    let mut output = io::stdout().lock();
    output.write_all(b"local lsp stdout\n")?;
    output.flush()?;
    std::thread::sleep(std::time::Duration::from_secs(30));
    Ok(())
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

fn write_frame(writer: &mut impl Write, value: &serde_json::Value) -> io::Result<()> {
    let body = serde_json::to_vec(value).map_err(io::Error::other)?;
    write!(writer, "Content-Length: {}\r\n\r\n", body.len())?;
    writer.write_all(&body)?;
    writer.flush()
}
