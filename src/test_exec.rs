use anyhow::{anyhow, Context, Result};
use indicatif::{ParallelProgressIterator, ProgressBar, ProgressStyle};
use rayon::{prelude::*, ThreadPoolBuilder};
use regex::Regex;
use walkdir::WalkDir;

use crate::config::ExecConfig;
use std::collections::BTreeMap;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::Duration;

pub fn test_all(
    exec_config: &ExecConfig,
    in_folder: PathBuf,
    out_folder: PathBuf,
    best_result_file: Option<PathBuf>,
) -> Result<()> {
    let results = run_test_multi_case(exec_config, &in_folder, &out_folder)?;

    match best_result_file {
        Some(best_result_file) => print_result_with_relative_score(results, best_result_file),
        None => print_result(results),
    }
}

pub fn print_result(mut results: Vec<(PathBuf, i64)>) -> Result<()> {
    println!("---------result-----------");

    results.sort_by(|a, b| (a.0).cmp(&(b.0)));

    let mut score_sum = 0;

    for (input_path, score) in results.iter() {
        score_sum += score;
        println!("{} : {}", input_path.to_str().unwrap(), score);
    }

    let max_case = results.iter().max_by(|a, b| (a.1).cmp(&(b.1))).unwrap();
    let min_case = results.iter().min_by(|a, b| (a.1).cmp(&(b.1))).unwrap();

    println!("score sum : {}", score_sum);
    println!(
        "score ave : {}",
        (score_sum as f64) / (results.len() as f64)
    );
    println!(
        "max case `{}` : {}",
        max_case.0.to_str().unwrap(),
        max_case.1
    );
    println!(
        "min case `{}` : {}",
        min_case.0.to_str().unwrap(),
        min_case.1
    );
    Ok(())
}

pub fn print_result_with_relative_score(
    mut results: Vec<(PathBuf, i64)>,
    best_result_file: PathBuf,
) -> Result<()> {
    let mut best_result_map: BTreeMap<PathBuf, i64> = if best_result_file.is_file() {
        let best_result_str = fs::read_to_string(&best_result_file).with_context(|| {
            format!(
                "could not read best-result-file `{}`",
                best_result_file.to_str().unwrap()
            )
        })?;

        serde_json::from_str(&best_result_str).with_context(|| {
            format!(
                "could not read best-result-file as json file `{}`",
                best_result_file.to_str().unwrap()
            )
        })?
    } else {
        BTreeMap::new()
    };

    println!("---------result-----------");

    results.sort_by(|a, b| (a.0).cmp(&(b.0)));
    let mut score_sum = 0;

    for (input_path, score) in results.iter_mut() {
        let best_score = match best_result_map.get(input_path) {
            Some(saved_score) => {
                if saved_score < score {
                    saved_score.clone()
                } else {
                    score.clone()
                }
            }
            None => score.clone(),
        }
        .clone();

        best_result_map.insert(input_path.clone(), best_score);

        let relative_score = (1e9 * best_score as f64 / (*score as f64)).round() as i64;

        score_sum += relative_score;
        *score = relative_score;
        println!(
            "{} : relative score {}",
            input_path.to_str().unwrap(),
            relative_score
        );
    }

    let serialized_str = serde_json::to_string_pretty(&best_result_map)
        .with_context(|| format!("could not serialize best result to json format"))?;

    let mut output_file = File::create(&best_result_file).with_context(|| {
        format!(
            "could not create best-result-file `{}`",
            best_result_file.to_str().unwrap()
        )
    })?;
    output_file
        .write_all(serialized_str.as_bytes())
        .with_context(|| {
            format!(
                "could not write best result to `{}`",
                best_result_file.to_str().unwrap()
            )
        })?;

    let max_case = results.iter().max_by(|a, b| (a.1).cmp(&(b.1))).unwrap();
    let min_case = results.iter().min_by(|a, b| (a.1).cmp(&(b.1))).unwrap();

    println!("score sum : {}", score_sum);
    println!(
        "score ave : {}",
        (score_sum as f64) / (results.len() as f64)
    );
    println!(
        "max case `{}` : {}",
        max_case.0.to_str().unwrap(),
        max_case.1
    );
    println!(
        "min case `{}` : {}",
        min_case.0.to_str().unwrap(),
        min_case.1
    );

    Ok(())
}

fn run_test_multi_case(
    exec_config: &ExecConfig,
    in_folder: &PathBuf,
    out_folder: &PathBuf,
) -> Result<Vec<(PathBuf, i64)>> {
    let score_regex =
        Regex::new(r"Score = (\d+)").with_context(|| format!("could not build regex instance"))?;

    let mut in_files = WalkDir::new(in_folder.clone())
        .into_iter()
        .filter_map(|e| e.ok())
        .map(|e| e.into_path())
        .filter(|e| e.is_file())
        .collect::<Vec<_>>();

    in_files.sort_by(|a, b| a.cmp(b));

    let th_pool = ThreadPoolBuilder::new()
        .num_threads(4)
        .build()
        .with_context(|| format!("could not build thread pool"))?;

    let results = th_pool.install(|| {
        let pb = ProgressBar::new(in_files.len() as u64);
        pb.enable_steady_tick(Duration::new(1, 0));
        pb.set_style(
            ProgressStyle::with_template(
                "[{elapsed_precise}] [{wide_bar:.cyan/blue}] {pos}/{len} cases ({eta})  ",
            )
            .unwrap()
            .progress_chars("#>-"),
        );

        let ps = in_files.par_iter().progress_with(pb).map(|entry| {
            run_test_single_case(exec_config, entry.clone(), &out_folder, &score_regex)
        });
        ps.collect::<Vec<Result<(PathBuf, i64)>>>()
    });

    results
        .into_iter()
        .collect::<Result<Vec<_>, _>>()
        .with_context(|| format!("failed in some case"))
}

fn run_test_single_case(
    exec_config: &ExecConfig,
    in_file: PathBuf,
    out_folder: &PathBuf,
    score_regex: &Regex,
) -> Result<(PathBuf, i64)> {
    let filename = in_file
        .file_name()
        .with_context(|| format!("could not get filename"))?;

    let input_file = File::open(in_file.clone())
        .with_context(|| format!("could not open input_file `{}`", in_file.to_str().unwrap()))?;

    let output_path = out_folder.join(filename);
    let output_file = File::create(output_path.clone()).with_context(|| {
        format!(
            "could not create output_file `{}`",
            output_path.to_str().unwrap()
        )
    })?;

    let stdin_pipe = Stdio::from(input_file);
    let stdout_pipe = Stdio::from(output_file);

    let output = Command::new(exec_config.tester_cmd.clone())
        .args(exec_config.tester_arg.clone())
        .arg(exec_config.solver_cmd.clone())
        .args(exec_config.solver_arg.clone())
        .stdin(stdin_pipe)
        .stdout(stdout_pipe)
        .stderr(Stdio::piped())
        .output()
        .with_context(|| format!("could not execute command"))?;

    let stderr_str = String::from_utf8_lossy(&output.stderr);
    match score_regex.captures(&stderr_str) {
        Some(caps) => {
            let score = (&caps[1]).parse().unwrap();
            Ok((in_file.clone(), score))
        }
        None => Err(anyhow!(
            "could not capture score_regex `{}`",
            in_file.to_str().unwrap()
        )),
    }
}
