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

pub fn pagination_query(limit: Option<usize>, offset: Option<usize>) -> String {
    let mut params = Vec::new();
    if let Some(l) = limit { params.push(format!("limit={}", l)); }
    if let Some(o) = offset { params.push(format!("offset={}", o)); }
    params.join("&")
}
