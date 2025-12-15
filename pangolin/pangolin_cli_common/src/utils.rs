use prettytable::{Table, Row, Cell};
use serde_json::Value;

pub fn print_table(headers: Vec<&str>, rows: Vec<Vec<String>>) {
    let mut table = Table::new();
    table.add_row(Row::new(headers.iter().map(|h| Cell::new(h)).collect()));
    
    for row in rows {
        table.add_row(Row::new(row.iter().map(|c| Cell::new(c)).collect()));
    }
    
    table.printstd();
}

pub fn format_json(value: &Value) -> String {
    serde_json::to_string_pretty(value).unwrap_or_else(|_| "Error formatting JSON".to_string())
}
