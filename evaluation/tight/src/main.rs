
#![feature(no_coverage)]
pub mod rusty_monitor;
pub use ntest::timeout;
pub mod data;
pub mod date;
pub mod expense;

use chrono::Datelike;
use clap::{App, AppSettings, Arg, SubCommand};
use colored::*;
use std::convert::TryInto;

const DATA_LOCATION: &str = ".tight/tight.json";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about("A simple expense tracker")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .subcommand(SubCommand::with_name("init")
                    .about("initialise an empty data file in the current directory"))
        .subcommand(SubCommand::with_name("add")
                    .about("add an income or expense")
                    .setting(AppSettings::SubcommandRequiredElseHelp)
                    .subcommand(SubCommand::with_name("tag")
                                .about("add a new tag")
                                .arg(Arg::with_name("name")
                                     .required(true)
                                     .help("the tag to add")))
                    .subcommand(SubCommand::with_name("expense")
                                .about("add an expense"))
                    .subcommand(SubCommand::with_name("income")
                                .about("add an income")))
        .subcommand(SubCommand::with_name("rm")
                    .about("remove an income or expense")
                    .setting(AppSettings::SubcommandRequiredElseHelp)
                    .subcommand(SubCommand::with_name("tag")
                                .about("remove a tag")
                                .arg(Arg::with_name("name")
                                     .required(true)
                                     .help("the name of the tag to remove")))
                    .subcommand(SubCommand::with_name("expense")
                                .about("remove an expense")
                                .arg(Arg::with_name("id")
                                     .required(true)
                                     .help("the ID of the expense to remove")))
                    .subcommand(SubCommand::with_name("income")
                                .about("remove an income")
                                .arg(Arg::with_name("id")
                                     .required(true)
                                     .help("the ID of the income to remove"))))
        .subcommand(SubCommand::with_name("grep")
                    .about("show expenses/incomes matching a regular expression")
                    .arg(Arg::with_name("regex")
                         .required(true)
                         .help("look for expenses and incomes with descriptions or tags matching this regular expression")))
        .subcommand(SubCommand::with_name("show")
                    .about("show an expense or income with a given ID")
                    .arg(Arg::with_name("id")
                         .required(true)
                         .help("the ID of the expense or income to show")))
        .subcommand(SubCommand::with_name("status")
                    .about("show position for day, week, month and year"))
        .subcommand(SubCommand::with_name("report")
                    .about("run a custom report. reserved for future usage"))
        .subcommand(SubCommand::with_name("verify")
                    .about("verify the integrity of the tight data file"))
        .subcommand(SubCommand::with_name("repair")
                    .about("attempt to repair the tight data file"))
        .subcommand(SubCommand::with_name("migrate")
                    .about("migrate the tight data file to the latest schema"))
        .get_matches();

    match matches.subcommand() {
        ("init", Some(_)) => {
            data::initialise(DATA_LOCATION)?;
        },

        ("add", Some(add_matches)) => {
            let mut data = data::Datafile::from_file(DATA_LOCATION)?;

            match add_matches.subcommand() {
                ("tag", Some(tag_matches)) => {
                    data.add_tag(tag_matches.value_of("name").unwrap().to_string());
                },

                ("expense", Some(_)) => {
                    let stdin = std::io::stdin();
                    let mut handle = stdin.lock();
                    let expense = expense::Expense::from_stdin(&mut handle, data.entries.len() as u64, false, &data.tags)?;
                    println!("added {}", expense);
                    data.insert(expense);
                },

                ("income", Some(_)) => {
                    let stdin = std::io::stdin();
                    let mut handle = stdin.lock();
                    let expense = expense::Expense::from_stdin(&mut handle, data.entries.len() as u64, true, &data.tags)?;
                    println!("added {}", expense);
                    data.insert(expense);
                },

                _ => unreachable!(),
            };

            data.save(DATA_LOCATION)?;
        },

        ("rm", Some(rm_matches)) => {
            let mut data = data::Datafile::from_file(DATA_LOCATION)?;

            match rm_matches.subcommand() {
                ("tag", Some(tag_matches)) => {
                    let name = tag_matches.value_of("name").unwrap();
                    for expense in &mut data.entries {
                        expense.remove_tags(name);
                    }

                    data.tags.retain(|tag| tag != name);
                },

                ("expense", Some(expense_matches)) => {
                    let id = expense_matches.value_of("id").unwrap().parse()?;
                    if let Some(expense) = data.find(id) {
                        if expense.amount() < 0 {
                            println!("removing {}", expense);
                            data.remove(id)?;
                        } else {
                            return Err("this is an income, not an expense".into());
                        }
                    } else {
                        return Err("couldn't find anything with that id".into());
                    }
                },

                ("income", Some(income_matches)) => {
                    let id = income_matches.value_of("id").unwrap().parse()?;
                    if let Some(income) = data.find(id) {
                        if income.amount() > 0 {
                            println!("removing {}", income);
                            data.remove(id)?;
                        } else {
                            return Err("this is an expense, not an income".into());
                        }
                    } else {
                        return Err("couldn't find anything with that id".into());
                    }
                },

                _ => unreachable!(),
            }

            data.save(DATA_LOCATION)?;
        },

        ("grep", Some(grep_matches)) => {
            let data = data::Datafile::from_file(DATA_LOCATION)?;
            let re = regex::Regex::new(grep_matches.value_of("regex").unwrap())?;
            for expense in data.entries {
                let mut print = false;
                for tag in expense.tags() {
                    print |= re.is_match(tag);
                }
                print |= re.is_match(expense.description());

                if print {
                    println!("{}", expense);
                }
            }
        },

        ("show", Some(show_matches)) => {
            let data = data::Datafile::from_file(DATA_LOCATION)?;
            let id = show_matches.value_of("id").unwrap().parse()?;
            if let Some(expense) = data.find(id) {
                println!("{}", expense);
            } else {
                return Err("couldn't find anything with that id".into());
            }
        }

        ("status", Some(_)) => {
            let data = data::Datafile::from_file(DATA_LOCATION)?;

            let now = chrono::Local::now();
            let year = now.year().try_into()?;
            let month = now.month().into();
            let day = now.day().into();
            let today = date::SimpleDate::from_ymd(year, month, day);

            let day_start = &today - &date::Duration::Day(1);
            let week_start = &today - &date::Duration::Week(1);
            let month_start = &today - &date::Duration::Month(1);
            let year_start = &today - &date::Duration::Year(1);

            let day_expenses = data.expenses_between(&day_start, &today);
            let week_expenses = data.expenses_between(&week_start, &today);
            let month_expenses = data.expenses_between(&month_start, &today);
            let year_expenses = data.expenses_between(&year_start, &today);

            let day_total: f64 = expense::calculate_spread(&day_expenses, &day_start, &date::Duration::Day(1));
            let week_total: f64 = expense::calculate_spread(&week_expenses, &week_start, &date::Duration::Week(1));
            let month_total: f64 = expense::calculate_spread(&month_expenses, &month_start, &date::Duration::Month(1));
            let year_total: f64 = expense::calculate_spread(&year_expenses, &year_start, &date::Duration::Year(1));

            let day_formatted = format!("{:.2}", day_total.abs());
            let week_formatted = format!("{:.2}", week_total.abs());
            let month_formatted = format!("{:.2}", month_total.abs());
            let year_formatted = format!("{:.2}", year_total.abs());

            print!("day: ");
            if day_total < 0.0 {
                println!("{}{}", "-$".red(), day_formatted.red());
            } else {
                println!("{}{}", "$".green(), day_formatted.green());
            }

            print!("week: ");
            if week_total < 0.0 {
                println!("{}{}", "-$".red(), week_formatted.red());
            } else {
                println!("{}{}", "$".green(), week_formatted.green());
            }

            print!("month: ");
            if month_total < 0.0 {
                println!("{}{}", "-$".red(), month_formatted.red());
            } else {
                println!("{}{}", "$".green(), month_formatted.green());
            }

            print!("year: ");
            if year_total < 0.0 {
                println!("{}{}", "-$".red(), year_formatted.red());
            } else {
                println!("{}{}", "$".green(), year_formatted.green());
            }
        }

        ("verify", Some(_)) => (),
        ("repair", Some(_)) => (),
        ("report", Some(_)) => (),
        ("migrate", Some(_)) => (),

        _ => unreachable!(),
    }

    Ok(())
}
