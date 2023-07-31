//! Namelist callbacks
//!

use std::error::Error;

use anyhow::anyhow;
use async_trait::async_trait;
use chrono::{Duration, NaiveDate};
use ntu_canoebot_attd::{
    calculate_sheet_name, get_config_type, refresh_sheet_cache, Sheet, SHEET_CACHE,
};
use ntu_canoebot_util::debug_println;
use serde::{Deserialize, Serialize};
use teloxide::{prelude::*, types::ParseMode};

use crate::{
    callback::message_from_callback_query,
    frame::{calendar_month_gen, calendar_year_gen, date_am_pm_navigation},
};

use super::{replace_with_whitespace, Callback, Date, HandleCallback};

use ntu_canoebot_config as config;

/// Callbacks for /namelist
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NameList {
    /// Get the namelist for a particular date
    /// Args:
    /// - Fetch date
    /// - time slot
    /// - cache refresh?
    Get(Date, bool, bool),

    MonthSelect(Date),

    YearSelect(Date),
}

#[async_trait]
impl HandleCallback for NameList {
    async fn handle_callback(
        &self,
        bot: Bot,
        query: CallbackQuery,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let msg = message_from_callback_query(&query)?;

        match self {
            NameList::Get(date, time_slot, refresh) => {
                replace_with_whitespace(bot.clone(), &msg, 2).await?;
                namelist_get(date.to_owned(), *time_slot, *refresh, bot, msg, true).await?
            }
            NameList::MonthSelect(date) => {
                // replace_with_whitespace(bot.clone(), &msg, 2).await?;
                let start = NaiveDate::from_ymd_opt(date.year, date.month, 1).unwrap();
                let days: Vec<Callback> = (0..31)
                    .into_iter()
                    .map(|d| {
                        Callback::NameList(NameList::Get(
                            (start + Duration::days(d as i64)).into(),
                            false,
                            false,
                        ))
                    })
                    .collect();

                let year = Callback::NameList(NameList::YearSelect(date.to_owned()));
                let prev =
                    Callback::NameList(NameList::MonthSelect((start - Duration::days(1)).into()));
                let next =
                    Callback::NameList(NameList::MonthSelect((start + Duration::days(33)).into()));

                let keyboard = calendar_month_gen((*date).into(), &days, year, next, prev, None);

                bot.edit_message_text(msg.chat.id, msg.id, "namelist")
                    .reply_markup(keyboard)
                    .await?;
            }
            NameList::YearSelect(date) => {
                // replace_with_whitespace(bot.clone(), &msg, 2).await?;
                let months: Vec<Callback> = (0..12)
                    .into_iter()
                    .map(|m| {
                        let month = Date {
                            year: date.year,
                            month: 1 + m,
                            day: 1,
                        };

                        Callback::NameList(NameList::MonthSelect(month))
                    })
                    .collect();

                let next = Callback::NameList(NameList::YearSelect(Date {
                    year: date.year + 1,
                    month: 1,
                    day: 1,
                }));
                let prev = Callback::NameList(NameList::YearSelect(Date {
                    year: date.year - 1,
                    month: 1,
                    day: 1,
                }));

                let keyboard = calendar_year_gen((*date).into(), &months, next, prev, None);

                bot.edit_message_text(msg.chat.id, msg.id, msg.text().unwrap_or(" "))
                    .reply_markup(keyboard)
                    .await?;
            }
        }

        Ok(())
    }
}

/// Perform a namelist get operation.
/// If an entry exists in cache and refresh is not triggered, it will pull data from the cache.
pub async fn namelist_get(
    date: Date,
    time_slot: bool,
    refresh: bool,
    bot: Bot,
    msg: &Message,
    is_callback: bool,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let date: NaiveDate = date.into();

    let config = get_config_type(date);
    let sheet_id = match config {
        ntu_canoebot_attd::Config::Old => *config::SHEETSCRAPER_OLD_ATTENDANCE_SHEET,
        ntu_canoebot_attd::Config::New => *config::SHEETSCRAPER_NEW_ATTENDANCE_SHEET,
    };

    debug_println!("date: {}\nusing {:?} config", date, config);

    let sheet_name = calculate_sheet_name(date);

    debug_println!("sheet name: {}", sheet_name);

    let sheet: Sheet = {
        // check if cache matches up with this sheet
        let read_lock = SHEET_CACHE.read().await;
        let in_cache = read_lock.contains_date(date.into());

        if in_cache {
            if refresh {
                drop(read_lock);
                refresh_sheet_cache(refresh).await.unwrap();
                SHEET_CACHE.read().await.clone()
            } else {
                read_lock.clone()
            }
        } else {
            let df = g_sheets::get_as_dataframe(sheet_id, Some(sheet_name)).await;
            df.try_into().ok().ok_or(anyhow!(""))?
        }
    };

    let list = sheet
        .get_names(date, time_slot)
        .await
        .unwrap_or(ntu_canoebot_attd::NameList {
            date,
            time: time_slot,
            names: Default::default(),
            boats: None,
            fetch_time: chrono::Local::now().naive_local()
        });

    // generate keyboard
    let prev = Callback::NameList(NameList::Get(
        {
            let d: NaiveDate = date.into();
            (d - Duration::days(1)).into()
        },
        time_slot,
        false,
    ));

    let next = Callback::NameList(NameList::Get(
        {
            let d: NaiveDate = date.into();
            (d + Duration::days(1)).into()
        },
        time_slot,
        false,
    ));

    let refresh = Callback::NameList(NameList::Get(date.into(), time_slot, true));

    let time = Callback::NameList(NameList::Get(date.into(), !time_slot, false));

    let calendar = Callback::NameList(NameList::MonthSelect(date.into()));
    let keyboard = date_am_pm_navigation(date, refresh, next, prev, time, calendar);

    let contents = format!("```\n{}```", list);

    match is_callback {
        true => {
            bot.edit_message_text(msg.chat.id, msg.id, contents)
                .reply_markup(keyboard)
                .parse_mode(ParseMode::MarkdownV2)
                .await?;
        }
        false => {
            bot.send_message(msg.chat.id, contents)
                .reply_markup(keyboard)
                .parse_mode(ParseMode::MarkdownV2)
                .await?;
        }
    }

    Ok(())
}
