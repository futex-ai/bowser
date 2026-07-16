use super::*;

#[test]
fn parses_check_command() {
    let cli = Cli::try_parse_from(["xtask", "check"]).unwrap();

    assert!(matches!(
        cli.command,
        Commands::Check(CheckArgs {
            include,
            exclude
        }) if include.is_empty() && exclude.is_empty()
    ));
}

#[test]
fn parses_check_include_and_exclude_phases() {
    let cli = Cli::try_parse_from([
        "xtask",
        "check",
        "--include",
        "rust-test,bowser",
        "--include",
        "workflow",
        "--exclude",
        "scripts",
    ])
    .unwrap();

    assert!(matches!(
        cli.command,
        Commands::Check(CheckArgs {
            include,
            exclude
        }) if include.as_slice() == [
            CheckPhaseArg::RustTest,
            CheckPhaseArg::Bowser,
            CheckPhaseArg::Workflow
        ] && exclude.as_slice() == [CheckPhaseArg::Scripts]
    ));
}

#[test]
fn parses_check_multiple_exclude_phases() {
    let cli = Cli::try_parse_from([
        "xtask",
        "check",
        "--exclude",
        "scripts",
        "--exclude",
        "workflow,bowser",
    ])
    .unwrap();

    assert!(matches!(
        cli.command,
        Commands::Check(CheckArgs {
            include,
            exclude
        }) if include.is_empty()
            && exclude.as_slice() == [
                CheckPhaseArg::Scripts,
                CheckPhaseArg::Workflow,
                CheckPhaseArg::Bowser
            ]
    ));
}

#[test]
fn parses_review_command() {
    let cli = Cli::try_parse_from(["xtask", "review"]).unwrap();

    assert!(matches!(cli.command, Commands::Review));
}

#[test]
fn parses_rust_trait_audit_command() {
    let cli = Cli::try_parse_from(["xtask", "rust-trait-audit"]).unwrap();

    assert!(matches!(cli.command, Commands::RustTraitAudit));
}

#[test]
fn parses_rust_source_audit_command() {
    let cli = Cli::try_parse_from(["xtask", "rust-source-audit"]).unwrap();

    assert!(matches!(cli.command, Commands::RustSourceAudit));
}

#[test]
fn parses_rust_file_length_lint_command() {
    let cli = Cli::try_parse_from(["xtask", "rust-file-length-lint"]).unwrap();

    assert!(matches!(
        cli.command,
        Commands::RustFileLengthLint(RustFileLengthLintArgs { all: false })
    ));
}

#[test]
fn parses_rust_file_length_lint_all_command() {
    let cli = Cli::try_parse_from(["xtask", "rust-file-length-lint", "--all"]).unwrap();

    assert!(matches!(
        cli.command,
        Commands::RustFileLengthLint(RustFileLengthLintArgs { all: true })
    ));
}

#[test]
fn pull_request_actionlint_download_retries_transient_failures() {
    let workflow = include_str!("../../../.github/workflows/pull-request.yml");
    let install_step = workflow
        .split("- name: Install actionlint")
        .nth(1)
        .expect("install actionlint step")
        .split("- name: Run repository checks")
        .next()
        .expect("next step after actionlint install");

    for expected in [
        "--retry 5",
        "--retry-delay 2",
        "--retry-max-time 120",
        "--retry-all-errors",
    ] {
        assert!(
            install_step.contains(expected),
            "missing actionlint curl retry option: {expected}"
        );
    }
}
