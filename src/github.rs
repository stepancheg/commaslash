use std::env;

#[cfg(test)]
pub(crate) fn is_github_actions() -> bool {
    env::var("GITHUB_ACTIONS").is_ok()
}
