pub(crate) struct Flags {
    pub dry_run: bool,
    pub quiet: bool,
    pub verbose: bool,
    pub json: bool,
    pub config: Option<String>,
}

impl Flags {
    pub fn from_cli(cli: &super::Cli) -> Self {
        Self {
            dry_run: cli.dry_run,
            quiet: cli.quiet,
            verbose: cli.verbose,
            json: cli.json,
            config: cli.config.clone(),
        }
    }

    pub fn config_path(&self) -> Option<&str> {
        self.config.as_deref()
    }
}
