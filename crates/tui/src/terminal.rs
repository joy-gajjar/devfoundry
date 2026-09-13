#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TerminalPaneState {
    pub terminal_id: Option<String>,
    pub attached: bool,
    pub input_lease_held: bool,
    pub input: Vec<u8>,
    pub output: Vec<u8>,
    pub output_offset: u64,
    pub gap: bool,
    pub focused: bool,
    pub rows: u16,
    pub columns: u16,
    pub error: Option<String>,
}

impl TerminalPaneState {
    pub fn apply_output(&mut self, offset: u64, bytes: &[u8], next_offset: u64) {
        if offset != self.output_offset {
            self.gap = true;
            return;
        }
        self.output.extend_from_slice(bytes);
        self.output_offset = next_offset;
        self.gap = false;
    }

    pub fn apply_output_gap(&mut self, next_offset: u64) {
        self.gap = true;
        self.output_offset = next_offset;
    }

    pub fn resize(&mut self, rows: u16, columns: u16) {
        self.rows = rows;
        self.columns = columns;
    }
}
