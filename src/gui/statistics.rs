use eframe::egui;

use crate::app::state::AppState;
use crate::gui::widgets;
use crate::services::calculations;

pub fn show(ui: &mut egui::Ui, state: &AppState) {
    ui.heading("Portfolio Statistics");
    ui.add_space(8.0);

    let portfolio = state.portfolio.lock().unwrap();
    let stats = calculations::portfolio_stats(&portfolio.investments);

    egui::Grid::new("stats_grid").num_columns(2).show(ui, |ui| {
        ui.label("Total invested");
        ui.label(format!("{} gp", widgets::format_gp(stats.total_invested as i64)));
        ui.end_row();

        ui.label("Current portfolio value");
        ui.label(format!("{} gp", widgets::format_gp(stats.current_value as i64)));
        ui.end_row();

        ui.label("Total profit / loss");
        widgets::gp_label(ui, stats.total_profit_loss);
        ui.end_row();

        ui.label("Percentage gain / loss");
        ui.label(format!("{:.2}%", stats.profit_loss_pct));
        ui.end_row();

        ui.label("Best investment");
        ui.label(
            stats
                .best_investment
                .as_ref()
                .map(|(n, pl)| format!("{n} ({} gp)", widgets::format_gp(*pl)))
                .unwrap_or_else(|| "-".into()),
        );
        ui.end_row();

        ui.label("Worst investment");
        ui.label(
            stats
                .worst_investment
                .as_ref()
                .map(|(n, pl)| format!("{n} ({} gp)", widgets::format_gp(*pl)))
                .unwrap_or_else(|| "-".into()),
        );
        ui.end_row();

        ui.label("Average holding period");
        ui.label(format!("{:.1} days", stats.average_holding_days));
        ui.end_row();
    });
}
