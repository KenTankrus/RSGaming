use chrono::Utc;
use eframe::egui;
use uuid::Uuid;

use crate::app::state::{AppState, GeLookupOutcome, GeLookupResult, ResolvedItem};
use crate::gui::widgets;
use crate::gui::UiState;
use crate::models::investment::{Game, Investment, InvestmentStatus};
use crate::services::{calculations, rs_item_lookup, storage};

pub fn show(ui: &mut egui::Ui, state: &AppState, ui_state: &mut UiState) {
    ui.heading("Investments");
    ui.add_space(8.0);

    egui::CollapsingHeader::new("Add new investment")
        .default_open(false)
        .show(ui, |ui| add_form(ui, state, ui_state));

    ui.add_space(12.0);
    ui.separator();

    let mut portfolio = state.portfolio.lock().unwrap();
    let mut dirty = false;
    let mut remove_id: Option<String> = None;

    egui::ScrollArea::vertical().show(ui, |ui| {
        for inv in portfolio.investments.iter_mut() {
            let is_editing = matches!(&ui_state.editing, Some((id, _)) if id == &inv.id);

            ui.group(|ui| {
                if is_editing {
                    let (_, form) = ui_state.editing.as_mut().unwrap();
                    let mut save = false;
                    let mut cancel = false;

                    egui::Grid::new(format!("edit_grid_{}", inv.id)).num_columns(2).show(ui, |ui| {
                        ui.label("Game");
                        ui.horizontal(|ui| {
                            ui.selectable_value(&mut form.game, Game::Rs3, "RS3");
                            ui.selectable_value(&mut form.game, Game::Osrs, "OSRS");
                        });
                        ui.end_row();

                        ui.label("Item name");
                        ui.text_edit_singleline(&mut form.item_name);
                        ui.end_row();

                        ui.label("Item ID");
                        ui.text_edit_singleline(&mut form.item_id);
                        ui.end_row();

                        ui.label("Purchase date");
                        ui.text_edit_singleline(&mut form.purchase_date);
                        ui.end_row();

                        ui.label("Purchase price (gp, each)");
                        ui.text_edit_singleline(&mut form.purchase_price);
                        ui.end_row();

                        ui.label("Quantity");
                        ui.text_edit_singleline(&mut form.quantity);
                        ui.end_row();

                        ui.label("Notes");
                        ui.text_edit_singleline(&mut form.notes);
                        ui.end_row();
                    });

                    if let Some(err) = &form.error {
                        ui.colored_label(egui::Color32::from_rgb(220, 60, 60), err);
                    }

                    ui.horizontal(|ui| {
                        if ui.button("Save").clicked() {
                            save = true;
                        }
                        if ui.button("Cancel").clicked() {
                            cancel = true;
                        }
                    });

                    if save {
                        form.error = None;
                        if form.item_name.trim().is_empty() {
                            form.error = Some("Item name can't be empty.".to_string());
                        }

                        let price = form.purchase_price.trim().parse::<u64>().ok().filter(|&p| p > 0);
                        if form.error.is_none() && price.is_none() {
                            form.error = Some("Purchase price must be a whole number greater than 0.".to_string());
                        }

                        let qty = form.quantity.trim().parse::<u64>().ok().filter(|&q| q > 0);
                        if form.error.is_none() && qty.is_none() {
                            form.error = Some("Quantity must be a whole number greater than 0.".to_string());
                        }

                        let purchase_date = if form.purchase_date.trim().is_empty() {
                            Some(inv.purchase_date)
                        } else {
                            chrono::NaiveDate::parse_from_str(form.purchase_date.trim(), "%Y-%m-%d")
                                .ok()
                                .and_then(|d| d.and_hms_opt(0, 0, 0))
                                .map(|dt| dt.and_utc())
                        };
                        if form.error.is_none() && purchase_date.is_none() {
                            form.error = Some("Couldn't read that date -- use YYYY-MM-DD.".to_string());
                        }

                        if form.error.is_none() {
                            let item_id: Option<u32> = form.item_id.trim().parse().ok();
                            inv.game = form.game;
                            inv.item_name = form.item_name.trim().to_string();
                            inv.item_id = item_id;
                            inv.purchase_date = purchase_date.unwrap();
                            inv.purchase_price = price.unwrap();
                            inv.quantity = qty.unwrap();
                            inv.notes = if form.notes.trim().is_empty() {
                                None
                            } else {
                                Some(form.notes.trim().to_string())
                            };
                            dirty = true;
                            ui_state.editing = None;
                        }
                    } else if cancel {
                        ui_state.editing = None;
                    }
                } else {
                    let metrics = calculations::metrics_for(inv);
                    ui.horizontal(|ui| {
                        ui.strong(&inv.item_name);
                        ui.weak(format!("[{}]", inv.game.label()));
                        ui.label(format!("qty {}", inv.quantity));
                        ui.label(format!("bought {}", inv.purchase_date.format("%Y-%m-%d")));
                        ui.label(format!("@ {} gp", widgets::format_gp(inv.purchase_price as i64)));
                        if let Some(p) = inv.current_price {
                            ui.label(format!("now {} gp", widgets::format_gp(p as i64)));
                        }
                        widgets::gp_label(ui, metrics.profit_loss);
                        ui.label(format!("{:.1}% - {} days", metrics.profit_loss_pct, metrics.days_held));
                    });
                    ui.horizontal(|ui| {
                        if ui.button("Edit").clicked() {
                            ui_state.editing = Some((inv.id.clone(), crate::gui::EditInvestmentForm::from_investment(inv)));
                        }
                        match inv.status {
                            InvestmentStatus::Active => {
                                if ui.button("Mark sold").clicked() {
                                    inv.status = InvestmentStatus::Sold;
                                    inv.sold_price = inv.current_price.or(Some(inv.purchase_price));
                                    inv.sold_date = Some(Utc::now());
                                    dirty = true;
                                }
                            }
                            InvestmentStatus::Sold => {
                                ui.label("Sold");
                            }
                        }
                        if ui.button("Delete").clicked() {
                            remove_id = Some(inv.id.clone());
                        }
                    });
                    if let Some(notes) = &inv.notes {
                        if !notes.is_empty() {
                            ui.label(format!("Notes: {notes}"));
                        }
                    }
                }
            });
        }
    });

    if let Some(id) = remove_id {
        portfolio.investments.retain(|i| i.id != id);
        dirty = true;
    }

    if dirty {
        if let Err(e) = storage::save_portfolio(&portfolio) {
            tracing::warn!("Failed to save portfolio: {e}");
        }
    }
}

fn add_form(ui: &mut egui::Ui, state: &AppState, ui_state: &mut UiState) {
    let form = &mut ui_state.new_investment;

    egui::Grid::new("new_investment_grid").num_columns(2).show(ui, |ui| {
        ui.label("Game");
        ui.horizontal(|ui| {
            ui.selectable_value(&mut form.game, Game::Rs3, "RS3");
            ui.selectable_value(&mut form.game, Game::Osrs, "OSRS");
        });
        ui.end_row();

        ui.label("Item name");
        ui.text_edit_singleline(&mut form.item_name);
        ui.end_row();

        ui.label("Item ID");
        ui.horizontal(|ui| {
            ui.text_edit_singleline(&mut form.item_id);
            if ui.button("Look up on GE").clicked() {
                spawn_lookup(state, form.game, form.item_id.trim(), form.item_name.trim());
            }
        });
        ui.end_row();

        ui.label("Purchase date");
        ui.text_edit_singleline(&mut form.purchase_date);
        ui.end_row();

        ui.label("Purchase price (gp, each)");
        ui.text_edit_singleline(&mut form.purchase_price);
        ui.end_row();

        ui.label("Quantity");
        ui.text_edit_singleline(&mut form.quantity);
        ui.end_row();

        ui.label("Notes");
        ui.text_edit_singleline(&mut form.notes);
        ui.end_row();
    });

    ui.label(egui::RichText::new("Format: YYYY-MM-DD").weak().small());
    if form.date_error {
        ui.colored_label(egui::Color32::from_rgb(220, 60, 60), "Couldn't read that date -- use YYYY-MM-DD.");
    }

    show_lookup_result(ui, state, form);

    ui.add_space(4.0);
    ui.label(
        egui::RichText::new(
            "RS3 and OSRS have separate item IDs -- \"Look up on GE\" will try to resolve an item name or confirm a typed item ID directly via the official RuneScape catalogue APIs. For anything else, find the item's ID on its wiki page and paste it in.",
        )
        .small(),
    );

    if ui.button("Add investment").clicked() {
        form.date_error = false;
        let price: u64 = form.purchase_price.trim().parse().unwrap_or(0);
        let qty: u64 = form.quantity.trim().parse().unwrap_or(0);
        let item_id: Option<u32> = form.item_id.trim().parse().ok();

        let purchase_date = if form.purchase_date.trim().is_empty() {
            Some(Utc::now())
        } else {
            chrono::NaiveDate::parse_from_str(form.purchase_date.trim(), "%Y-%m-%d")
                .ok()
                .and_then(|d| d.and_hms_opt(0, 0, 0))
                .map(|dt| dt.and_utc())
        };

        let Some(purchase_date) = purchase_date else {
            form.date_error = true;
            return;
        };

        if !form.item_name.trim().is_empty() && price > 0 && qty > 0 {
            let investment = Investment {
                id: Uuid::new_v4().to_string(),
                item_name: form.item_name.trim().to_string(),
                item_id,
                game: form.game,
                purchase_date,
                purchase_price: price,
                quantity: qty,
                notes: if form.notes.trim().is_empty() {
                    None
                } else {
                    Some(form.notes.trim().to_string())
                },
                status: InvestmentStatus::Active,
                current_price: None,
                previous_price: None,
                last_updated: None,
                sold_price: None,
                sold_date: None,
            };

            let mut portfolio = state.portfolio.lock().unwrap();
            portfolio.investments.push(investment);
            if let Err(e) = storage::save_portfolio(&portfolio) {
                tracing::warn!("Failed to save portfolio: {e}");
            }
            drop(portfolio);

            *form = Default::default();
            *state.ge_lookup.lock().unwrap() = None;
        }
    }
}

/// Kicks off a GE lookup: if an item ID was typed, confirms it directly
/// against the RS3 catalogue; otherwise tries the item name against the
/// built-in known-items shortlist. Result comes back asynchronously into
/// `state.ge_lookup`, read by `show_lookup_result` on the next frame(s).
fn spawn_lookup(state: &AppState, game: Game, id_text: &str, name_text: &str) {
    let ge_lookup = state.ge_lookup.clone();

    if let Ok(id) = id_text.parse::<u32>() {
        *ge_lookup.lock().unwrap() = Some(GeLookupResult {
            query: id_text.to_string(),
            outcome: Err("Looking up...".to_string()),
        });

        state.tokio_handle.spawn(async move {
            let outcome = match game {
                Game::Rs3 => rs_item_lookup::lookup_rs3_item(id).await,
                Game::Osrs => rs_item_lookup::lookup_osrs_item(id).await,
            }
            .map(|d| ResolvedItem {
                id: d.id,
                name: d.name,
                current_price: d.current_price,
                members: d.members,
            })
            .map(GeLookupOutcome::Single);
            *ge_lookup.lock().unwrap() = Some(GeLookupResult {
                query: id.to_string(),
                outcome: outcome.map_err(|e| e.to_string()),
            });
        });
        return;
    }

    if name_text.is_empty() {
        *ge_lookup.lock().unwrap() = Some(GeLookupResult {
            query: String::new(),
            outcome: Err("Type an item name or numeric ID first.".to_string()),
        });
        return;
    }

    let query = name_text.to_string();
    *ge_lookup.lock().unwrap() = Some(GeLookupResult {
        query: query.clone(),
        outcome: Err("Looking up...".to_string()),
    });

    state.tokio_handle.spawn(async move {
        let outcome = match game {
            Game::Rs3 => rs_item_lookup::search_rs3_items(&query).await,
            Game::Osrs => rs_item_lookup::search_osrs_items(&query).await,
        }
        .map(|results| match results.len() {
            0 => Err(format!(
                "No {} item matched \"{query}\". Try a more specific name or paste the item ID from the wiki.",
                game.label()
            )),
            1 => Ok(GeLookupOutcome::Single(ResolvedItem {
                id: results[0].id,
                name: results[0].name.clone(),
                current_price: results[0].current_price,
                members: results[0].members,
            })),
            _ => Ok(GeLookupOutcome::Multiple(
                results
                    .into_iter()
                    .map(|d| ResolvedItem {
                        id: d.id,
                        name: d.name,
                        current_price: d.current_price,
                        members: d.members,
                    })
                    .collect(),
            )),
        })
        .map_err(|e| e.to_string())
        .and_then(|outcome| outcome);

        *ge_lookup.lock().unwrap() = Some(GeLookupResult {
            query,
            outcome,
        });
    });
}

fn show_lookup_result(ui: &mut egui::Ui, state: &AppState, form: &mut crate::gui::NewInvestmentForm) {
    let mut apply: Option<(u32, String)> = None;

    {
        let lookup = state.ge_lookup.lock().unwrap();
        if let Some(result) = lookup.as_ref() {
            // Only show the result if it still corresponds to what's
            // currently typed in the form -- if the user changed the item
            // ID/name after triggering a lookup but before it came back,
            // an old result for a different query would be confusing at
            // best and misleading at worst.
            let still_relevant = result.query == form.item_id.trim()
                || result.query.eq_ignore_ascii_case(form.item_name.trim());

            if still_relevant {
                match &result.outcome {
                    Ok(GeLookupOutcome::Single(item)) => {
                        ui.horizontal(|ui| {
                            ui.colored_label(
                                egui::Color32::from_rgb(60, 179, 113),
                                format!(
                                    "Found: {} (id {}){}{}",
                                    item.name,
                                    item.id,
                                    item.current_price
                                        .map(|p| format!(" -- current price {} gp", widgets::format_gp(p as i64)))
                                        .unwrap_or_default(),
                                    if item.members { " -- members" } else { "" }
                                ),
                            );
                            if ui.small_button("Use this").clicked() {
                                apply = Some((item.id, item.name.clone()));
                            }
                        });
                    }
                    Ok(GeLookupOutcome::Multiple(items)) => {
                        ui.label(format!("Found {} results. Click one to use:", items.len()));
                        let mut displayed = 0;
                        for item in items.iter().take(8) {
                            displayed += 1;
                            ui.horizontal(|ui| {
                                ui.colored_label(
                                    egui::Color32::from_rgb(60, 179, 113),
                                    format!(
                                        "{} (id {}){}{}",
                                        item.name,
                                        item.id,
                                        item.current_price
                                            .map(|p| format!(" -- current price {} gp", widgets::format_gp(p as i64)))
                                            .unwrap_or_default(),
                                        if item.members { " -- members" } else { "" }
                                    ),
                                );
                                if ui.small_button("Use this").clicked() {
                                    apply = Some((item.id, item.name.clone()));
                                }
                            });
                        }
                        if items.len() > displayed {
                            ui.label(format!("...and {} more results", items.len() - displayed));
                        }
                    }
                    Err(msg) => {
                        ui.label(msg);
                    }
                }
            }
        }
    }

    if let Some((id, name)) = apply {
        form.item_id = id.to_string();
        if form.item_name.trim().is_empty() {
            form.item_name = name;
        }
        *state.ge_lookup.lock().unwrap() = None;
    }
}
