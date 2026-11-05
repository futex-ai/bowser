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

#[test]
fn release_builds_checkout_the_verified_commit() {
    let workflow = include_str!("../../../.github/workflows/release.yml");

    assert!(
        workflow.contains("commit_sha: ${{ steps.release.outputs.commit_sha }}"),
        "verify job must export the checked-out release commit"
    );
    assert!(
        workflow.contains("commit_sha=$(git rev-parse HEAD)"),
        "release validation must capture the exact checked-out commit"
    );
    assert!(
        workflow.contains("ref: ${{ needs.verify.outputs.commit_sha }}"),
        "build jobs must checkout the commit validated by the verify job"
    );
    assert!(
        workflow.contains("format('refs/tags/{0}', inputs.tag)"),
        "manual releases must resolve an explicitly qualified tag ref"
    );
}

#[test]
fn built_cli_smokes_verify_the_reported_package_version() {
    let main_workflow = include_str!("../../../.github/workflows/main.yml");
    let release_workflow = include_str!("../../../.github/workflows/release.yml");

    assert!(
        main_workflow.contains("target/release/bowser --version"),
        "main must smoke the built CLI version flag"
    );
    assert!(
        release_workflow.contains("\"target/$TARGET/release/bowser\" --version"),
        "release builds must smoke the packaged CLI version flag"
    );
    assert!(
        release_workflow.contains("bowser $VERSION"),
        "release builds must compare the CLI output with the verified tag version"
    );
}
