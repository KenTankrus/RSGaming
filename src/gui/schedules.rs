use chrono::{NaiveTime, Timelike, Weekday};
use eframe::egui;
use uuid::Uuid;

use crate::app::state::AppState;
use crate::gui::{EventKindChoice, FrequencyKind, NewEventRuleForm, NewScheduleForm, UiState};
use crate::models::investment::InvestmentStatus;
use crate::models::schedule::{EventKind, EventRule, Frequency, TimeSchedule};
use crate::services::storage;

const ALL_WEEKDAYS: [Weekday; 7] = [
    Weekday::Mon,
    Weekday::Tue,
    Weekday::Wed,
    Weekday::Thu,
    Weekday::Fri,
    Weekday::Sat,
    Weekday::Sun,
];

fn weekday_label(w: Weekday) -> &'static str {
    match w {
        Weekday::Mon => "Monday",
        Weekday::Tue => "Tuesday",
        Weekday::Wed => "Wednesday",
        Weekday::Thu => "Thursday",
        Weekday::Fri => "Friday",
        Weekday::Sat => "Saturday",
        Weekday::Sun => "Sunday",
    }
}

pub fn show(ui: &mut egui::Ui, state: &AppState, ui_state: &mut UiState) {
    ui.heading("Notification Scheduling");
    ui.add_space(4.0);
    ui.label(
        egui::RichText::new("All times below are your PC's local time, using 24-hour format (military time). Do not use AM/PM.")
            .weak()
            .small(),
    );
    ui.add_space(8.0);

    show_time_schedules(ui, state, &mut ui_state.new_schedule);

    ui.add_space(16.0);
    ui.separator();
    ui.add_space(8.0);

    show_event_rules(ui, state, &mut ui_state.new_event_rule);
}

fn show_time_schedules(ui: &mut egui::Ui, state: &AppState, form: &mut NewScheduleForm) {
    ui.label(egui::RichText::new("Time-based summaries").strong());

    let mut store = state.schedules.lock().unwrap();
    let mut dirty = false;
    let mut remove_id: Option<String> = None;

    for sched in store.time_schedules.iter_mut() {
        ui.group(|ui| {
            ui.horizontal(|ui| {
                if ui.checkbox(&mut sched.enabled, &sched.name).changed() {
                    dirty = true;
                }
                ui.label(frequency_label(sched.frequency));
                if ui.button("Remove").clicked() {
                    remove_id = Some(sched.id.clone());
                }
            });

            ui.horizontal(|ui| {
                if sched.frequency.uses_day_of_week() {
                    ui.label("Day:");
                    egui::ComboBox::from_id_salt(format!("day_{}", sched.id))
                        .selected_text(weekday_label(sched.day_of_week))
                        .show_ui(ui, |ui| {
                            for day in ALL_WEEKDAYS {
                                if ui
                                    .selectable_value(&mut sched.day_of_week, day, weekday_label(day))
                                    .changed()
                                {
                                    dirty = true;
                                }
                            }
                        });
                }

                if sched.frequency.uses_time_of_day() {
                    ui.label("Time:");
                    let mut hour = sched.time_of_day.hour() as i32;
                    let mut minute = sched.time_of_day.minute() as i32;
                    let mut changed = false;
                    // NOTE: DragValue::range() matches egui ~0.28+; if your
                    // installed version predates that, use .clamp_range()
                    // instead -- not checked live in this session.
                    changed |= ui.add(egui::DragValue::new(&mut hour).range(0..=23)).changed();
                    ui.label(":");
                    changed |= ui
                        .add(egui::DragValue::new(&mut minute).range(0..=59))
                        .changed();
                    if changed {
                        if let Some(t) = NaiveTime::from_hms_opt(hour as u32, minute as u32, 0) {
                            sched.time_of_day = t;
                            dirty = true;
                        }
                    }
                }
            });
        });
    }

    if let Some(id) = remove_id {
        store.time_schedules.retain(|s| s.id != id);
        dirty = true;
    }

    ui.add_space(8.0);
    egui::CollapsingHeader::new("+ Add a summary schedule")
        .default_open(false)
        .show(ui, |ui| {
            egui::Grid::new("new_schedule_grid").num_columns(2).show(ui, |ui| {
                ui.label("Name");
                ui.text_edit_singleline(&mut form.name);
                ui.label(
                    egui::RichText::new("Leave blank to name the alert after the selected condition.")
                        .weak()
                        .small(),
                );
                ui.end_row();

                ui.label("Frequency");
                egui::ComboBox::from_id_salt("new_schedule_kind")
                    .selected_text(form.kind.label())
                    .show_ui(ui, |ui| {
                        for kind in FrequencyKind::ALL {
                            ui.selectable_value(&mut form.kind, kind, kind.label());
                        }
                    });
                ui.end_row();

                if matches!(form.kind, FrequencyKind::EveryXDays | FrequencyKind::EveryXHours) {
                    let unit = if form.kind == FrequencyKind::EveryXHours { "hours" } else { "days" };
                    ui.label(format!("Every X {unit}"));
                    let mut x = form.every_x as i32;
                    if ui.add(egui::DragValue::new(&mut x).range(1..=365)).changed() {
                        form.every_x = x.max(1) as u32;
                    }
                    ui.end_row();
                }

                if matches!(form.kind, FrequencyKind::Weekly | FrequencyKind::EveryOtherWeek) {
                    ui.label("Day of week");
                    egui::ComboBox::from_id_salt("new_schedule_day")
                        .selected_text(weekday_label(form.day_of_week))
                        .show_ui(ui, |ui| {
                            for day in ALL_WEEKDAYS {
                                ui.selectable_value(&mut form.day_of_week, day, weekday_label(day));
                            }
                        });
                    ui.end_row();
                }

                if form.kind != FrequencyKind::EveryXHours {
                    ui.label("Time");
                    ui.horizontal(|ui| {
                        let mut hour = form.hour as i32;
                        let mut minute = form.minute as i32;
                        if ui.add(egui::DragValue::new(&mut hour).range(0..=23)).changed() {
                            form.hour = hour.max(0) as u32;
                        }
                        ui.label(":");
                        if ui.add(egui::DragValue::new(&mut minute).range(0..=59)).changed() {
                            form.minute = minute.max(0) as u32;
                        }
                    });
                    ui.end_row();
                }
            });

            if ui.button("Add schedule").clicked() {
                let frequency = match form.kind {
                    FrequencyKind::Daily => Frequency::Daily,
                    FrequencyKind::EveryXDays => Frequency::EveryXDays(form.every_x.max(1)),
                    FrequencyKind::Weekly => Frequency::Weekly,
                    FrequencyKind::EveryOtherWeek => Frequency::EveryOtherWeek,
                    FrequencyKind::Monthly => Frequency::Monthly,
                    FrequencyKind::EveryXHours => Frequency::EveryXHours(form.every_x.max(1)),
                };
                let time_of_day = NaiveTime::from_hms_opt(form.hour, form.minute, 0)
                    .unwrap_or_else(|| NaiveTime::from_hms_opt(0, 0, 0).expect("0:00:00 is always valid"));

                store.time_schedules.push(TimeSchedule {
                    id: Uuid::new_v4().to_string(),
                    name: if form.name.trim().is_empty() {
                        "Portfolio summary".to_string()
                    } else {
                        form.name.trim().to_string()
                    },
                    frequency,
                    time_of_day,
                    day_of_week: form.day_of_week,
                    enabled: true,
                    last_run: None,
                });
                dirty = true;
                *form = NewScheduleForm::default();
            }
        });

    if dirty {
        if let Err(e) = storage::save_schedules(&store) {
            tracing::warn!("Failed to save schedules: {e}");
        }
    }
}

fn show_event_rules(ui: &mut egui::Ui, state: &AppState, form: &mut NewEventRuleForm) {
    ui.label(egui::RichText::new("Event-based alerts").strong());

    // Lock ordering convention (must match services::scheduler): schedules
    // before portfolio, everywhere, to avoid a lock-order-inversion
    // deadlock between this (GUI/main thread) and the background scheduler
    // task, which can run concurrently.
    let mut store = state.schedules.lock().unwrap();
    let portfolio = state.portfolio.lock().unwrap();
    let mut dirty = false;
    let mut remove_id: Option<String> = None;

    for rule in store.event_rules.iter_mut() {
        ui.horizontal(|ui| {
            if ui.checkbox(&mut rule.enabled, &rule.name).changed() {
                dirty = true;
            }
            let target = match &rule.item_id {
                None => "all active investments".to_string(),
                Some(id) => portfolio
                    .investments
                    .iter()
                    .find(|i| &i.id == id)
                    .map(|i| i.item_name.clone())
                    .unwrap_or_else(|| "(deleted investment)".to_string()),
            };
            ui.label(format!("{} -- {}", rule.kind.describe(), target));
            if ui.button("Remove").clicked() {
                remove_id = Some(rule.id.clone());
            }
        });
    }
    if let Some(id) = remove_id {
        store.event_rules.retain(|r| r.id != id);
        dirty = true;
    }

    ui.add_space(8.0);
    egui::CollapsingHeader::new("+ Add an alert")
        .default_open(false)
        .show(ui, |ui| {
            egui::Grid::new("new_event_rule_grid").num_columns(2).show(ui, |ui| {
                ui.label("Name");
                ui.text_edit_singleline(&mut form.name);
                ui.end_row();

                ui.label("Alert type");
                egui::ComboBox::from_id_salt("new_rule_kind")
                    .selected_text(form.kind.label())
                    .show_ui(ui, |ui| {
                        for kind in EventKindChoice::ALL {
                            ui.selectable_value(&mut form.kind, kind, kind.label());
                        }
                    });
                ui.end_row();

                ui.label("Applies to");
                let target_label = match &form.target_investment_id {
                    None => "All active investments".to_string(),
                    Some(id) => portfolio
                        .investments
                        .iter()
                        .find(|i| &i.id == id)
                        .map(|i| i.item_name.clone())
                        .unwrap_or_else(|| "All active investments".to_string()),
                };
                egui::ComboBox::from_id_salt("new_rule_target")
                    .selected_text(target_label)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut form.target_investment_id, None, "All active investments");
                        for inv in portfolio.investments.iter().filter(|i| i.status == InvestmentStatus::Active) {
                            ui.selectable_value(
                                &mut form.target_investment_id,
                                Some(inv.id.clone()),
                                &inv.item_name,
                            );
                        }
                    });
                ui.end_row();

                ui.label(threshold_label(form.kind));
                ui.text_edit_singleline(&mut form.threshold);
                ui.end_row();
            });

            if let Some(err) = &form.error {
                ui.colored_label(egui::Color32::from_rgb(220, 60, 60), err);
            }

            if ui.button("Add alert").clicked() {
                match build_event_kind(form.kind, &form.threshold) {
                    Ok(kind) => {
                        store.event_rules.push(EventRule {
                            id: Uuid::new_v4().to_string(),
                            name: if form.name.trim().is_empty() {
                                kind.describe()
                            } else {
                                form.name.trim().to_string()
                            },
                            item_id: form.target_investment_id.clone(),
                            kind,
                            enabled: true,
                            last_triggered: None,
                        });
                        dirty = true;
                        *form = NewEventRuleForm::default();
                    }
                    Err(e) => form.error = Some(e),
                }
            }
        });

    if dirty {
        if let Err(e) = storage::save_schedules(&store) {
            tracing::warn!("Failed to save schedules: {e}");
        }
    }
}

fn threshold_label(kind: EventKindChoice) -> &'static str {
    match kind {
        EventKindChoice::PriceAbove | EventKindChoice::PriceBelow => "Single-item price (gp)",
        EventKindChoice::ValueAbove | EventKindChoice::ValueBelow => "Whole investment value (gp)",
        EventKindChoice::ProfitAboveGp => "Profit (gp)",
        EventKindChoice::ProfitAbovePercent | EventKindChoice::SignificantMove => "Percent (e.g. 10 for 10%)",
    }
}

fn build_event_kind(kind: EventKindChoice, threshold_text: &str) -> Result<EventKind, String> {
    let text = threshold_text.trim();
    match kind {
        EventKindChoice::PriceAbove => text
            .parse::<u64>()
            .map(EventKind::PriceAbove)
            .map_err(|_| "Enter a whole number of gp.".to_string()),
        EventKindChoice::PriceBelow => text
            .parse::<u64>()
            .map(EventKind::PriceBelow)
            .map_err(|_| "Enter a whole number of gp.".to_string()),
        EventKindChoice::ValueAbove => text
            .parse::<u64>()
            .map(EventKind::ValueAbove)
            .map_err(|_| "Enter a whole number of gp.".to_string()),
        EventKindChoice::ValueBelow => text
            .parse::<u64>()
            .map(EventKind::ValueBelow)
            .map_err(|_| "Enter a whole number of gp.".to_string()),
        EventKindChoice::ProfitAboveGp => text
            .parse::<i64>()
            .map(EventKind::ProfitAboveGp)
            .map_err(|_| "Enter a whole number of gp (can be negative).".to_string()),
        EventKindChoice::ProfitAbovePercent => text
            .parse::<f64>()
            .map(EventKind::ProfitAbovePercent)
            .map_err(|_| "Enter a percentage, e.g. 15 or 15.5.".to_string()),
        EventKindChoice::SignificantMove => text
            .parse::<f64>()
            .map(EventKind::SignificantMove)
            .map_err(|_| "Enter a percentage, e.g. 10 or 10.5.".to_string()),
    }
}

fn frequency_label(f: Frequency) -> String {
    match f {
        Frequency::Daily => "Daily".to_string(),
        Frequency::EveryXDays(x) => format!("Every {x} days"),
        Frequency::Weekly => "Weekly".to_string(),
        Frequency::EveryOtherWeek => "Every other week".to_string(),
        Frequency::Monthly => "Monthly".to_string(),
        Frequency::EveryXHours(x) => format!("Every {x} hours"),
    }
}
