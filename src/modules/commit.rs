use doc::Docs;

use crate::{impl_display_for_module, impl_module, modules::Module, sync::OnceLock};

static COMMIT: OnceLock<Commit> = OnceLock::new();

#[derive(Debug, Docs)]
pub struct Commit {
    #[doc = "Author"]
    pub author: &'static str,
    #[doc = "Author's email"]
    pub email: &'static str,
    #[doc = "Creation date, `like Sep 03 2026, 20:09:11`"]
    pub date: &'static str,
    #[doc = "Creation (small) date, like `Sep 03 2026`"]
    pub date_small: &'static str,
    #[doc = "Commit sha1 hash"]
    pub sha: &'static str,
    #[doc = "Commit sha1 hash truncated to 7 chars"]
    pub sha_short: &'static str,
    #[doc = "Message"]
    pub message: &'static str,
    #[doc = "Number of changed files"]
    pub files: usize,
    #[doc = "Number of added lines"]
    pub added: usize,
    #[doc = "Number of deleted lines"]
    pub deleted: usize,
    #[doc = "Total number of changed lines"]
    pub total: usize,
}

impl Module for Commit {
    fn new() -> Self {
        Self {
            author: env!("COMMIT_AUTHOR"),
            email: env!("COMMIT_EMAIL"),
            date: env!("COMMIT_DATE"),
            date_small: env!("COMMIT_DATE_SMALL"),
            sha: env!("COMMIT_SHA"),
            sha_short: env!("COMMIT_SHA_SMALL"),
            message: env!("COMMIT_MESSAGE"),
            files: env!("COMMIT_FILES").parse::<usize>().expect("Unreachable"),
            added: env!("COMMIT_ADDED").parse::<usize>().expect("Unreachable"),
            deleted: env!("COMMIT_DELETED")
                .parse::<usize>()
                .expect("Unreachable"),
            total: env!("COMMIT_TOTAL").parse::<usize>().expect("Unreachable"),
        }
    }

    fn get() -> &'static Self {
        COMMIT.get_or_init(|| Self::new())
    }

    fn key(&self) -> &'static str {
        "Commit"
    }

    fn title(&self) -> &'static str {
        "{message} [{date-small}] @ \x1b[32m+{added} \x1b[31m-{deleted}\x1b[0m"
    }

    fn string_name(&self) -> &'static str {
        "commit"
    }

    impl_module!(
        author, email, date, date_small, sha, sha_short, message, files, added, deleted, total
    );
}

impl_display_for_module!(Commit);
