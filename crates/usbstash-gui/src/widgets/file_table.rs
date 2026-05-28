#![allow(dead_code)]
use usbstash_core::StashEntry;

/// Sort column for the file table.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortColumn {
    Name,
    Size,
    Date,
}

impl SortColumn {
    /// Cycle to the next sort column.
    pub fn next(self) -> Self {
        match self {
            SortColumn::Name => SortColumn::Size,
            SortColumn::Size => SortColumn::Date,
            SortColumn::Date => SortColumn::Name,
        }
    }

    /// Get the display label for this column.
    pub fn label(&self) -> &'static str {
        match self {
            SortColumn::Name => "Name",
            SortColumn::Size => "Size",
            SortColumn::Date => "Date",
        }
    }
}

/// Sort entries by the given column (ascending).
pub fn sort_entries(entries: &[StashEntry], column: SortColumn) -> Vec<&StashEntry> {
    let mut refs: Vec<&StashEntry> = entries.iter().collect();
    refs.sort_by(|a, b| match column {
        SortColumn::Name => {
            let name_a = a.path().rsplit('/').next().unwrap_or(a.path());
            let name_b = b.path().rsplit('/').next().unwrap_or(b.path());
            name_a.to_lowercase().cmp(&name_b.to_lowercase())
        }
        SortColumn::Size => a.size().cmp(&b.size()),
        SortColumn::Date => a.modified_at().cmp(&b.modified_at()),
    });
    refs
}

/// Format bytes into a human-readable string.
pub fn format_bytes(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else if bytes < 1024 * 1024 * 1024 {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.2} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}

/// Format a unix timestamp to a date string.
pub fn format_timestamp(timestamp: u64) -> String {
    let secs = timestamp;
    let days = secs / 86400;
    let years = 1970 + days / 365;
    let day_of_year = days % 365;
    let month = day_of_year / 30 + 1;
    let day = day_of_year % 30 + 1;
    format!("{:04}-{:02}-{:02}", years, month.min(12), day.min(28))
}

/// Extract the filename from a path.
pub fn filename_from_path(path: &str) -> &str {
    path.rsplit('/').next().unwrap_or(path)
}

/// Extract the file extension from a path.
pub fn extension_from_path(path: &str) -> &str {
    let name = filename_from_path(path);
    if let Some(pos) = name.rfind('.') {
        &name[pos + 1..]
    } else {
        ""
    }
}

/// Render the file table widget.
///
/// Displays entries in a table with columns for name, size, and date.
/// Clicking a row selects it.
///
/// Returns `true` if the selection changed.
pub fn show(
    ui: &mut egui::Ui,
    entries: &[StashEntry],
    sort: &mut SortColumn,
    selected: &mut Option<String>,
) -> bool {
    let mut changed = false;

    // Header row with clickable column headers for sorting
    ui.horizontal(|ui| {
        ui.heading("Name");
        ui.add_space(20.0);
        ui.heading("Size");
        ui.add_space(20.0);
        ui.heading("Date");
    });
    ui.separator();

    // Sort entries
    let sorted = sort_entries(entries, *sort);

    // Render rows
    for entry in &sorted {
        let is_selected = selected.as_ref().is_some_and(|s| s == entry.path());
        let name = filename_from_path(entry.path());

        let response = ui.horizontal(|ui| {
            if is_selected {
                ui.label(egui::RichText::new(name).strong());
            } else {
                ui.label(name);
            }
            ui.add_space(20.0);
            ui.label(format_bytes(entry.size()));
            ui.add_space(20.0);
            ui.small(format_timestamp(entry.modified_at()));
        });

        if response.response.clicked() {
            *selected = Some(entry.path().to_string());
            changed = true;
        }
    }

    changed
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_entry(path: &str, size: u64, modified_at: u64) -> StashEntry {
        StashEntry::new(
            uuid::Uuid::new_v4(),
            path.to_string(),
            0,
            modified_at,
            size,
            "application/octet-stream".to_string(),
            vec![],
        )
    }

    #[test]
    fn test_sort_by_name() {
        let entries = vec![
            make_entry("zebra.txt", 100, 0),
            make_entry("alpha.txt", 50, 0),
            make_entry("Beta.txt", 75, 0),
        ];
        let sorted = sort_entries(&entries, SortColumn::Name);
        let names: Vec<&str> = sorted
            .iter()
            .map(|e| filename_from_path(e.path()))
            .collect();
        assert_eq!(names, vec!["alpha.txt", "Beta.txt", "zebra.txt"]);
    }

    #[test]
    fn test_sort_by_size() {
        let entries = vec![
            make_entry("large.txt", 1000, 0),
            make_entry("small.txt", 10, 0),
            make_entry("medium.txt", 100, 0),
        ];
        let sorted = sort_entries(&entries, SortColumn::Size);
        let sizes: Vec<u64> = sorted.iter().map(|e| e.size()).collect();
        assert_eq!(sizes, vec![10, 100, 1000]);
    }

    #[test]
    fn test_sort_by_date() {
        let entries = vec![
            make_entry("old.txt", 100, 1000),
            make_entry("new.txt", 100, 3000),
            make_entry("mid.txt", 100, 2000),
        ];
        let sorted = sort_entries(&entries, SortColumn::Date);
        let dates: Vec<u64> = sorted.iter().map(|e| e.modified_at()).collect();
        assert_eq!(dates, vec![1000, 2000, 3000]);
    }

    #[test]
    fn test_sort_column_next() {
        assert_eq!(SortColumn::Name.next(), SortColumn::Size);
        assert_eq!(SortColumn::Size.next(), SortColumn::Date);
        assert_eq!(SortColumn::Date.next(), SortColumn::Name);
    }

    #[test]
    fn test_sort_column_label() {
        assert_eq!(SortColumn::Name.label(), "Name");
        assert_eq!(SortColumn::Size.label(), "Size");
        assert_eq!(SortColumn::Date.label(), "Date");
    }

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(0), "0 B");
        assert_eq!(format_bytes(512), "512 B");
        assert_eq!(format_bytes(1024), "1.0 KB");
        assert_eq!(format_bytes(1536), "1.5 KB");
        assert_eq!(format_bytes(1048576), "1.0 MB");
        assert_eq!(format_bytes(1073741824), "1.00 GB");
    }

    #[test]
    fn test_filename_from_path() {
        assert_eq!(filename_from_path("file.txt"), "file.txt");
        assert_eq!(filename_from_path("docs/file.txt"), "file.txt");
        assert_eq!(filename_from_path("a/b/c/file.txt"), "file.txt");
        assert_eq!(filename_from_path(""), "");
    }

    #[test]
    fn test_extension_from_path() {
        assert_eq!(extension_from_path("file.txt"), "txt");
        assert_eq!(extension_from_path("file.tar.gz"), "gz");
        assert_eq!(extension_from_path("noextension"), "");
        assert_eq!(extension_from_path("docs/file.pdf"), "pdf");
    }

    #[test]
    fn test_format_timestamp() {
        let ts = format_timestamp(1705276800);
        assert!(ts.starts_with("2024"));
    }

    #[test]
    fn test_empty_entries_sort() {
        let entries: Vec<StashEntry> = vec![];
        let sorted = sort_entries(&entries, SortColumn::Name);
        assert!(sorted.is_empty());
    }

    #[test]
    fn test_single_entry_sort() {
        let entries = vec![make_entry("only.txt", 42, 12345)];
        let sorted = sort_entries(&entries, SortColumn::Name);
        assert_eq!(sorted.len(), 1);
        assert_eq!(sorted[0].path(), "only.txt");
    }
}
