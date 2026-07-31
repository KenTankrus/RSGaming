use std::sync::{Arc, Mutex};
use std::time::Duration as StdDuration;

use chrono::Utc;

use crate::models::investment::Investment;
use crate::models::portfolio::Portfolio;
use crate::models::schedule::{EventKind, EventRule, ScheduleStore};
use crate::models::settings::AppSettings;
use crate::services::{calculations, ge_client::GeClient, portfolio as portfolio_service, storage, telegram};

/// Runs for the lifetime of the app: periodically refreshes GE prices,
/// checks due time-based schedules and event-based alert rules, and sends
/// Telegram notifications. Meant to be spawned once via `tokio::spawn` (or
/// a runtime handle) and kept running alongside the GUI.
pub async fn run(
    portfolio: Arc<Mutex<Portfolio>>,
    settings: Arc<Mutex<AppSettings>>,
    schedules: Arc<Mutex<ScheduleStore>>,
) {
    let client = GeClient::new();
    let mut last_price_refresh: Option<chrono::DateTime<Utc>> = None;

    // Run once immediately on startup, rather than waiting for the first
    // tick interval to elapse -- otherwise the dashboard sits with no
    // current prices at all for up to a full tick (previously 60s) after
    // every launch, which looked like a bug even though it wasn't one.
    tick_once(&portfolio, &settings, &schedules, &client, &mut last_price_refresh).await;

    let mut tick = tokio::time::interval(StdDuration::from_secs(crate::constants::SCHEDULER_TICK_SECONDS));
    tick.tick().await; // consume the immediate first tick eagerly

    loop {
        tick.tick().await;
        tick_once(&portfolio, &settings, &schedules, &client, &mut last_price_refresh).await;
    }
}

async fn tick_once(
    portfolio: &Arc<Mutex<Portfolio>>,
    settings: &Arc<Mutex<AppSettings>>,
    schedules: &Arc<Mutex<ScheduleStore>>,
    client: &GeClient,
    last_price_refresh: &mut Option<chrono::DateTime<Utc>>,
) {
    // --- Refresh prices, respecting the configured interval ---
    let refresh_interval_minutes = settings.lock().unwrap().refresh_interval_minutes;
    let due_for_refresh = match *last_price_refresh {
        None => true,
        Some(last) => Utc::now() - last >= chrono::Duration::minutes(refresh_interval_minutes as i64),
    };

    if due_for_refresh {
        match portfolio_service::refresh_prices(portfolio, client).await {
            Ok(summary) => {
                tracing::info!(
                    "Price refresh: {} updated, {} missing item ID, {} unresolved",
                    summary.updated,
                    summary.missing_item_id,
                    summary.unresolved
                );
                *last_price_refresh = Some(Utc::now());
                let pf = portfolio.lock().unwrap();
                if let Err(e) = storage::save_portfolio(&pf) {
                    tracing::warn!("Failed to persist portfolio: {e}");
                }
            }
            Err(e) => tracing::warn!("Price refresh failed: {e}"),
        }
    }

    // --- Time-based schedules (checked against local wall-clock time,
    // since schedules are configured with a local time-of-day / weekday) ---
    let mut outgoing: Vec<String> = Vec::new();
    {
        let mut store = schedules.lock().unwrap();
        let pf = portfolio.lock().unwrap();
        let now_local = chrono::Local::now();

        for sched in store.time_schedules.iter_mut() {
            if sched.is_due(now_local) {
                let stats = calculations::portfolio_stats(&pf.investments);
                outgoing.push(format_stats_message(&sched.name, &stats));
                sched.last_run = Some(Utc::now());
            }
        }

        drop(pf);
        if let Err(e) = storage::save_schedules(&store) {
            tracing::warn!("Failed to persist schedules: {e}");
        }
    }

    // --- Event-based rules ---
    {
        let mut store = schedules.lock().unwrap();
        let pf = portfolio.lock().unwrap();

        for rule in store.event_rules.iter_mut() {
            if !rule.enabled {
                continue;
            }
            for inv in pf.investments.iter() {
                if let Some(target_id) = &rule.item_id {
                    if target_id != &inv.id {
                        continue;
                    }
                }
                if let Some(msg) = evaluate_event(rule, inv) {
                    outgoing.push(msg);
                    rule.last_triggered = Some(Utc::now());
                }
            }
        }

        drop(pf);
        if let Err(e) = storage::save_schedules(&store) {
            tracing::warn!("Failed to persist schedules: {e}");
        }
    }

    // --- Send notifications (Telegram today; other channels can be added
    // here later without touching the logic above) ---
    if !outgoing.is_empty() {
        let cfg = settings.lock().unwrap().telegram.clone();
        for msg in outgoing {
            if let Err(e) = telegram::send_message(&cfg, &msg).await {
                tracing::warn!("Failed to send Telegram notification: {e}");
            }
        }
    }
}

fn evaluate_event(rule: &EventRule, inv: &Investment) -> Option<String> {
    let metrics = calculations::metrics_for(inv);
    let current_price = inv.current_price?;

    let hit = match rule.kind {
        EventKind::PriceAbove(target) => current_price > target,
        EventKind::PriceBelow(target) => current_price < target,
        EventKind::ValueAbove(target) => metrics.current_value > target,
        EventKind::ValueBelow(target) => metrics.current_value < target,
        EventKind::ProfitAboveGp(target) => metrics.profit_loss > target,
        EventKind::ProfitAbovePercent(target) => metrics.profit_loss_pct > target,
        EventKind::SignificantMove(pct_threshold) => match inv.previous_price {
            Some(prev) if prev > 0 => {
                let change = ((current_price as f64 - prev as f64) / prev as f64) * 100.0;
                change.abs() >= pct_threshold
            }
            _ => false,
        },
    };

    let reported_value = match rule.kind {
        EventKind::PriceAbove(_) | EventKind::PriceBelow(_) | EventKind::SignificantMove(_) => current_price,
        EventKind::ValueAbove(_) | EventKind::ValueBelow(_) => metrics.current_value,
        EventKind::ProfitAboveGp(_) | EventKind::ProfitAbovePercent(_) => metrics.current_value,
    };

    if hit {
        Some(format!(
            "{}: {} triggered -- now {} gp (P/L {:.1}%)",
            rule.name, inv.item_name, reported_value, metrics.profit_loss_pct
        ))
    } else {
        None
    }
}

fn format_stats_message(schedule_name: &str, stats: &calculations::PortfolioStats) -> String {
    format!(
        "{}\nInvested: {} gp\nCurrent value: {} gp\nP/L: {} gp ({:.1}%)",
        schedule_name, stats.total_invested, stats.current_value, stats.total_profit_loss, stats.profit_loss_pct
    )
}
