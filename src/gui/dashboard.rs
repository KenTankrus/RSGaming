use eframe::egui;

use crate::app::state::AppState;
use crate::gui::widgets;
use crate::models::investment::InvestmentStatus;
use crate::services::calculations;

pub fn show(ui: &mut egui::Ui, state: &AppState) {
    let portfolio = state.portfolio.lock().unwrap();
    let stats = calculations::portfolio_stats(&portfolio.investments);
    let active_count = portfolio
        .investments
        .iter()
        .filter(|i| i.status == InvestmentStatus::Active)
        .count();
    let missing_id_count = portfolio
        .investments
        .iter()
        .filter(|i| i.status == InvestmentStatus::Active && i.item_id.is_none())
        .count();

    ui.heading("Portfolio Dashboard");
    ui.add_space(8.0);

    ui.horizontal(|ui| {
        summary_card(ui, "Total Invested", &format!("{} gp", widgets::format_gp(stats.total_invested as i64)));
        summary_card(ui, "Current Value", &format!("{} gp", widgets::format_gp(stats.current_value as i64)));
        ui.vertical(|ui| {
            ui.label("Profit / Loss");
            widgets::gp_label(ui, stats.total_profit_loss);
            ui.label(format!("{:.2}%", stats.profit_loss_pct));
        });
    });

    ui.add_space(12.0);
    ui.horizontal(|ui| {
        if ui.button("Refresh prices now").clicked() {
            spawn_manual_refresh(state);
        }
        if let Some(status) = state.refresh_status.lock().unwrap().as_ref() {
            ui.label(status);
        }
    });

    if missing_id_count > 0 {
        ui.colored_label(
            egui::Color32::from_rgb(230, 160, 40),
            format!(
                "{missing_id_count} active investment(s) have no item ID set, so their price can't \
                 be pulled automatically -- edit them from the Investments tab and use \"Look up on GE\".",
            ),
        );
    }

    ui.add_space(12.0);
    ui.separator();
    ui.label(format!("Active investments: {active_count}"));
    ui.add_space(8.0);

    egui::ScrollArea::vertical().show(ui, |ui| {
        for inv in portfolio
            .investments
            .iter()
            .filter(|i| i.status == InvestmentStatus::Active)
        {
            let m = calculations::metrics_for(inv);
            ui.horizontal(|ui| {
                ui.label(&inv.item_name);
                ui.weak(format!("[{}]", inv.game.label()));
                ui.label(format!("qty {}", inv.quantity));
                ui.label(format!("bought {}", inv.purchase_date.format("%Y-%m-%d")));
                match inv.current_price {
                    Some(p) => {
                        ui.label(format!("cur {} gp", widgets::format_gp(p as i64)));
                        widgets::gp_label(ui, m.profit_loss);
                        ui.label(format!("{:.1}%", m.profit_loss_pct));
                    }
                    None => {
                        let reason = if inv.item_id.is_none() {
                            "no item ID"
                        } else {
                            "not yet refreshed"
                        };
                        ui.colored_label(egui::Color32::from_rgb(230, 160, 40), format!("no price yet ({reason})"));
                    }
                }
            });
        }
    });

    if active_count > 0 {
        ui.add_space(8.0);
        let last_updated = portfolio
            .investments
            .iter()
            .filter(|i| i.status == InvestmentStatus::Active)
            .filter_map(|i| i.last_updated)
            .max();
        match last_updated {
            Some(t) => ui.weak(format!(
                "Prices last updated {}",
                t.with_timezone(&chrono::Local).format("%Y-%m-%d %H:%M:%S")
            )),
            None => ui.weak("Prices haven't been refreshed yet."),
        };
    }
}

fn spawn_manual_refresh(state: &AppState) {
    let portfolio = state.portfolio.clone();
    let status = state.refresh_status.clone();
    *status.lock().unwrap() = Some("Refreshing...".to_string());

    state.tokio_handle.spawn(async move {
        let client = crate::services::ge_client::GeClient::new();
        let msg = match crate::services::portfolio::refresh_prices(&portfolio, &client).await {
            Ok(summary) => {
                if let Err(e) = crate::services::storage::save_portfolio(&portfolio.lock().unwrap()) {
                    tracing::warn!("Failed to persist portfolio: {e}");
                }
                format!(
                    "Updated {} item(s){}{}",
                    summary.updated,
                    if summary.missing_item_id > 0 {
                        format!(", {} with no item ID", summary.missing_item_id)
                    } else {
                        String::new()
                    },
                    if summary.unresolved > 0 {
                        format!(", {} the API didn't return a price for", summary.unresolved)
                    } else {
                        String::new()
                    }
                )
            }
            Err(e) => format!("Refresh failed: {e}"),
        };
        *status.lock().unwrap() = Some(msg);
    });
}

fn summary_card(ui: &mut egui::Ui, label: &str, value: &str) {
    ui.vertical(|ui| {
        ui.label(label);
        ui.heading(value);
    });
    ui.add_space(24.0);
}
