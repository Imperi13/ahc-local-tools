use anyhow::{anyhow, Context, Result};
use indicatif::{ParallelProgressIterator, ProgressBar, ProgressStyle};
use rayon::{prelude::*, ThreadPoolBuilder};
use regex::Regex;
use walkdir::WalkDir;

use crate::config::ExecConfig;
use std::fs::File;
use std::path::PathBuf;
use std::process::{Command, Stdio};

pub fn test_all(exec_config: &ExecConfig, in_folder: PathBuf, out_folder: PathBuf) -> Result<()> {
    let th_pool = ThreadPoolBuilder::new()
        .num_threads(4)
        .build()
        .with_context(|| format!("could not build thread pool"))?;

    let re =
        Regex::new(r"Score = (\d+)").with_context(|| format!("could not build regex instance"))?;

    let results = th_pool.install(|| {
        let mut files = WalkDir::new(in_folder.clone())
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .collect::<Vec<_>>();

        files.sort_by(|a, b| a.path().cmp(b.path()));

        let pb = ProgressBar::new(files.len() as u64);
        pb.set_style(ProgressStyle::with_template("[{elapsed_precise}] [{wide_bar:.cyan/blue}] {pos}/{len} cases ({eta})  ")
            .unwrap()
            .progress_chars("#>-"));

        let ps = files
            .par_iter()
            .progress_with(pb)
            .map(|entry| {
                test_single_case(exec_config, entry.clone().into_path(), &out_folder, &re)
            });
        ps.collect::<Vec<Result<(PathBuf, i64)>>>()
    });

    println!("---------result-----------");

    let mut results = results
        .into_iter()
        .collect::<Result<Vec<_>, _>>()
        .with_context(|| format!("failed in some case"))?;

    results.sort_by(|a, b| (a.0).cmp(&(b.0)));

    let mut score_sum = 0;

    for (input_path, score) in results.iter() {
        score_sum += score;
        println!("{} : {}", input_path.to_str().unwrap(), score);
    }

    println!("score sum : {}", score_sum);
    println!(
        "score ave : {}",
        (score_sum as f64) / (results.len() as f64)
    );
    Ok(())
}

fn test_single_case(
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
