use std::convert::TryFrom;
use std::collections::HashMap;
use std::path::PathBuf;

extern crate hyperpolyglot;
use hyperpolyglot::{get_language_breakdown, Detection, Language};

use colored::*;
use colorsys::Rgb;

#[path = "repo.rs"]
mod repo;

pub fn construct_language_summary() -> Vec<(Option<Language>, f64)> {
	let top_level_path = repo::top_level_repo_path();

	if let Some(top_level_path) = top_level_path {
		let language_breakdown: HashMap<&'static str, Vec<(Detection, PathBuf)>> = get_language_breakdown(top_level_path);

		// https://github.com/monkslc/hyperpolyglot/blob/40f091679b94057ec925f7f8925e2960d1d9dbf2/src/bin/main.rs#L121-L133
		let total_file_count = language_breakdown.iter()
										.fold(0, |acc, (_, files)| acc + files.len()) as f64;
		let mut lang_summary: Vec<(Option<Language>, f64)> = Vec::new();
		for (language, files) in language_breakdown {
			let percentage = ((files.len() * 100) as f64) / total_file_count;
			let language_struct: Option<Language> = match Language::try_from(language) {
				Ok(lang) => Some(lang),
				Err(_) => None,
			};
			lang_summary.push((language_struct, percentage))
		}

		// Sort by percentage (assuming our percentages are never NaN
		lang_summary.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

		lang_summary
	} else {
		vec![]
	}
}

pub fn print_language_summary(top_n: usize, language_summary: Vec<(Option<Language>, f64)>) {
	for (language, percentage) in language_summary.iter().take(top_n) {
		if let Some(language) = language {
			if let Some(lang_colour) = language.color {
				let rgb = Rgb::from_hex_str(lang_colour).unwrap();
				let r = rgb.red().round() as u8;
				let g = rgb.green().round() as u8;
				let b = rgb.blue().round() as u8;
				let language_summary_str = format!("{:>6.2}%  {}", percentage, language.name).truecolor(r, g, b);
				println!("{}", language_summary_str);
			} else {
				println!("{:>6.2}%  {}", percentage, language.name);
			}
		} else {
			println!("{:>6.2}%  UNKNOWN LANGUAGE", percentage);
		}
	}
}
