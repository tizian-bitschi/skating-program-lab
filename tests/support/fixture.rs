#[derive(Debug)]
pub struct Row<'a> {
    pub fields: Vec<(&'a str, &'a str)>,
}

impl<'a> Row<'a> {
    pub fn get(&self, name: &str) -> &'a str {
        self.fields
            .iter()
            .find(|(field, _)| *field == name)
            .unwrap_or_else(|| panic!("unknown field {name}"))
            .1
    }
}

pub fn rows<'a>(text: &'a str, schema: &'a str) -> Vec<Row<'a>> {
    let mut lines = text.lines();
    assert_eq!(lines.next(), Some(schema), "TSV header/schema mismatch");
    let columns: Vec<_> = schema.split('\t').collect();
    lines
        .map(|line| {
            let cells: Vec<_> = line.split('\t').collect();
            assert_eq!(cells.len(), columns.len(), "column count: {line}");
            Row {
                fields: columns.iter().copied().zip(cells).collect(),
            }
        })
        .collect()
}

// Printed signed hundredths only: no float parsing, rounding or production scorer.
pub fn decimal(text: &str) -> Result<i64, &'static str> {
    let unsigned = text.strip_prefix('-').unwrap_or(text);
    let (whole, fraction) = unsigned.split_once('.').ok_or("missing decimal point")?;
    if whole.is_empty()
        || !whole.bytes().all(|b| b.is_ascii_digit())
        || fraction.len() != 2
        || !fraction.bytes().all(|b| b.is_ascii_digit())
    {
        return Err("expected exact printed hundredths");
    }
    let value = whole
        .parse::<i64>()
        .ok()
        .and_then(|w| w.checked_mul(100))
        .and_then(|w| fraction.parse::<i64>().ok().and_then(|f| w.checked_add(f)))
        .ok_or("decimal overflow")?;
    Ok(if text.starts_with('-') { -value } else { value })
}

pub fn cents(text: &str, location: &str) -> i64 {
    decimal(text).unwrap_or_else(|error| panic!("{location}: {text:?}: {error}"))
}
