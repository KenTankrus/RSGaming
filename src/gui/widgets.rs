use eframe::egui;

/// Draws a gp amount colored green (profit), red (loss), or neutral (zero).
pub fn gp_label(ui: &mut egui::Ui, amount: i64) {
    let text = format!("{} gp", format_gp(amount));
    let color = if amount > 0 {
        egui::Color32::from_rgb(60, 179, 113)
    } else if amount < 0 {
        egui::Color32::from_rgb(220, 60, 60)
    } else {
        ui.visuals().text_color()
    };
    ui.colored_label(color, text);
}

/// Formats a signed integer with thousands separators, e.g. -1234567 -> "-1,234,567".
pub fn format_gp(amount: i64) -> String {
    let neg = amount < 0;
    let digits: String = amount.unsigned_abs().to_string();

    let mut grouped = String::new();
    for (i, c) in digits.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            grouped.push(',');
        }
        grouped.push(c);
    }
    let grouped: String = grouped.chars().rev().collect();

    if neg {
        format!("-{grouped}")
    } else {
        grouped
    }
}
