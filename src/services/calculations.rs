use chrono::Utc;

use crate::models::investment::{Investment, InvestmentStatus};

#[derive(Debug, Clone, Copy, Default)]
pub struct InvestmentMetrics {
    pub total_cost: u64,
    pub current_value: u64,
    pub profit_loss: i64,
    pub profit_loss_pct: f64,
    pub days_held: i64,
}

/// Computes total cost, current value, profit/loss, profit/loss %, and days
/// held for a single investment. For sold items, "current value" is based
/// on the recorded sale price rather than the (now irrelevant) live price.
pub fn metrics_for(inv: &Investment) -> InvestmentMetrics {
    let total_cost = inv.purchase_price * inv.quantity;

    let price_now = match inv.status {
        InvestmentStatus::Sold => inv.sold_price.unwrap_or(inv.purchase_price),
        InvestmentStatus::Active => inv.current_price.unwrap_or(inv.purchase_price),
    };
    let current_value = price_now * inv.quantity;

    let profit_loss = current_value as i64 - total_cost as i64;
    let profit_loss_pct = if total_cost > 0 {
        (profit_loss as f64 / total_cost as f64) * 100.0
    } else {
        0.0
    };

    let end_date = inv.sold_date.unwrap_or_else(Utc::now);
    let days_held = (end_date - inv.purchase_date).num_days();

    InvestmentMetrics {
        total_cost,
        current_value,
        profit_loss,
        profit_loss_pct,
        days_held,
    }
}

#[derive(Debug, Clone, Default)]
pub struct PortfolioStats {
    pub total_invested: u64,
    pub current_value: u64,
    pub total_profit_loss: i64,
    pub profit_loss_pct: f64,
    pub best_investment: Option<(String, i64)>,
    pub worst_investment: Option<(String, i64)>,
    pub average_holding_days: f64,
}

/// Aggregates portfolio-wide statistics across every investment (active and
/// sold), per the "Portfolio Statistics" section of the spec.
pub fn portfolio_stats(investments: &[Investment]) -> PortfolioStats {
    let mut stats = PortfolioStats::default();
    if investments.is_empty() {
        return stats;
    }

    let mut total_days = 0i64;
    let mut best: Option<(String, i64)> = None;
    let mut worst: Option<(String, i64)> = None;

    for inv in investments {
        let m = metrics_for(inv);
        stats.total_invested += m.total_cost;
        stats.current_value += m.current_value;
        stats.total_profit_loss += m.profit_loss;
        total_days += m.days_held;

        if best.as_ref().is_none_or(|(_, pl)| m.profit_loss > *pl) {
            best = Some((inv.item_name.clone(), m.profit_loss));
        }
        if worst.as_ref().is_none_or(|(_, pl)| m.profit_loss < *pl) {
            worst = Some((inv.item_name.clone(), m.profit_loss));
        }
    }

    stats.profit_loss_pct = if stats.total_invested > 0 {
        (stats.total_profit_loss as f64 / stats.total_invested as f64) * 100.0
    } else {
        0.0
    };
    stats.average_holding_days = total_days as f64 / investments.len() as f64;
    stats.best_investment = best;
    stats.worst_investment = worst;

    stats
}
