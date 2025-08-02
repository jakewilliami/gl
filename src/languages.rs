use super::opts::GitLogOptions;
use super::repo;
use colored::*;
use colorsys::Rgb;
use hyperpolyglot::{Detection, Language, get_language_breakdown};
use std::collections::HashMap;
use std::convert::TryFrom;
use std::path::PathBuf;

pub struct LanguageSummary {
    language: Option<Language>,
    prevalence_percentage: f64,
    colour: Option<UnsignedRGB>,
}

pub struct UnsignedRGB {
    r: u8,
    g: u8,
    b: u8,
}

pub fn construct_language_summary() -> Vec<LanguageSummary> {
    let top_level_path = repo::top_level_repo_path();

    if let Some(top_level_path) = top_level_path {
        let language_breakdown: HashMap<&'static str, Vec<(Detection, PathBuf)>> =
            get_language_breakdown(top_level_path);

        // https://github.com/monkslc/hyperpolyglot/blob/40f091679b94057ec925f7f8925e2960d1d9dbf2/src/bin/main.rs#L121-L133
        let total_file_count = language_breakdown
            .iter()
            .fold(0, |acc, (_, files)| acc + files.len()) as f64;
        let mut lang_summary: Vec<LanguageSummary> = Vec::new();
        for (language, files) in language_breakdown {
            // Get the prevalence of this language in the repo
            let percentage = ((files.len() * 100) as f64) / total_file_count;

            // Get the language from the database
            let language_struct = Language::try_from(language).ok();

            // Get colour information for this language
            let rgb: Option<UnsignedRGB> = match language_struct {
                Some(lang) => {
                    if let Some(lang_colour_str) = lang.color {
                        let rgb = Rgb::from_hex_str(lang_colour_str).unwrap();
                        Some(UnsignedRGB {
                            r: rgb.red().round() as u8,
                            g: rgb.green().round() as u8,
                            b: rgb.blue().round() as u8,
                        })
                    } else {
                        None
                    }
                }
                None => None,
            };

            // Push our resulting summary data to the vector
            lang_summary.push(LanguageSummary {
                language: language_struct,
                prevalence_percentage: percentage,
                colour: rgb,
            });
        }

        // Sort by percentage (assuming our percentages are never NaN
        lang_summary.sort_by(|a, b| {
            b.prevalence_percentage
                .partial_cmp(&a.prevalence_percentage)
                .unwrap()
        });

        lang_summary
    } else {
        // If there is no top-level path (i.e., we may not be in a git repo), return an empty vector,
        // as we cannot determine any language information
        // TODO: allow user to optionally input any directory, not just assume it's a git one
        vec![]
    }
}

pub fn print_language_summary(
    top_n: usize,
    languages_summary: Vec<LanguageSummary>,
    opts: &GitLogOptions,
) {
    for language_summary in languages_summary.iter().take(top_n) {
        // Check if the language was present in the database
        if let Some(language) = language_summary.language {
            if opts.colour {
                if let Some(lang_colour) = &language_summary.colour {
                    let language_summary_str = format!(
                        "{:>6.2}%  {}",
                        language_summary.prevalence_percentage, language.name
                    )
                    .truecolor(lang_colour.r, lang_colour.g, lang_colour.b);
                    println!("{language_summary_str}");
                } else {
                    println!(
                        "{:>6.2}%  {}",
                        language_summary.prevalence_percentage, language.name
                    );
                }
            } else {
                println!(
                    "{:>6.2}%  {}",
                    language_summary.prevalence_percentage, language.name
                );
            }
        } else {
            println!(
                "{:>6.2}%  UNKNOWN LANGUAGE",
                language_summary.prevalence_percentage
            );
        }
    }
}
