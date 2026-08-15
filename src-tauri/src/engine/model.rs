// STATUS: DIAMANT VGT SUPREME

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Info,
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    RemoveSafe,
    RemoveStrict,
    RemoveMaximum,
    NormalizeStrict,
    NormalizeMaximum,
    ReportOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rule {
    pub name: &'static str,
    pub category: &'static str,
    pub severity: Severity,
    pub action: Action,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Safe,
    Strict,
    Maximum,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EngineResult {
    pub output: String,
    pub removed_count: usize,
    pub normalized_count: usize,
    pub change_count: usize,
    pub changes: Vec<EngineChange>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EngineChange {
    pub source_index: usize,
    pub output_index: usize,
    pub before: String,
    pub after: String,
    pub kind: &'static str,
}
