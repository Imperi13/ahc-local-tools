use anyhow::{Context, Result};
use rayon::{prelude::*, ThreadPoolBuilder};
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

    th_pool.install(|| {
        let files = WalkDir::new(in_folder.clone())
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .collect::<Vec<_>>();

        println!("{:?}", files);
        let ps = files
            .into_par_iter()
            .map(|entry| test_single_case(exec_config, entry.into_path(), &out_folder));
        let result = ps.collect::<Vec<Result<(PathBuf, i64)>>>();

        println!("{:?}", result);
    });
    Ok(())
}

fn test_single_case(
    exec_config: &ExecConfig,
    in_file: PathBuf,
    out_folder: &PathBuf,
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

    println!("{:?}", output);
    Ok((in_file, 0))
}
