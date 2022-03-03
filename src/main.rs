mod config;

use std::cmp::{self, Ordering};

use git2::{BranchType, Repository, RepositoryState};
use term::color::{self, Color};

use crate::config::Config;

const COLOR_A: Color = color::WHITE;
const COLOR_B: Color = color::BLUE;

struct Repo {
    name: String,
    branches: Vec<Branch>,
    state: RepositoryState,
}

struct Branch {
    name: String,
    is_head: bool,
}

fn main() -> Result<(), anyhow::Error> {
    let config = Config::default()?;

    let mut t = term::stdout().expect("cannot fail");

    let mut repos = Vec::new();
    for path in config.repos() {
        let repo = Repository::open(path)?;
        let mut branches = repo
            .branches(Some(BranchType::Local))?
            .map(|result| {
                let (branch, _branch_type) = result?;
                let is_head = branch.is_head();
                let name = branch
                    .name()
                    .map_err(|e| anyhow::anyhow!("5: {}", e))?
                    .ok_or_else(|| anyhow::anyhow!("6"))?
                    .to_string();
                Ok(Branch { name, is_head })
            })
            .collect::<Result<Vec<_>, anyhow::Error>>()?;
        branches.sort_by(|a, b| {
            if a.is_head {
                return Ordering::Less;
            }
            if b.is_head {
                return Ordering::Greater;
            }
            a.name.cmp(&b.name)
        });
        let name = path
            .file_name()
            .expect("TODO")
            .to_str()
            .expect("TODO")
            .to_string();
        repos.push(Repo {
            name,
            branches,
            state: repo.state(),
        });
    }

    let mut len_longest_repo_name = 0;
    for path in config.repos() {
        let name = path.file_name().expect("TODO").to_str().expect("TODO");
        len_longest_repo_name = cmp::max(name.len(), len_longest_repo_name);
    }

    let mut len_longest_name = 0;
    for repo in repos.iter() {
        for branch in &repo.branches {
            len_longest_name = cmp::max(branch.name.len(), len_longest_name);
        }
    }

    const LEN_LONGEST_STATE: usize = 23; // apply-mailbox-or-bebase

    let len = len_longest_repo_name + 5 + len_longest_name + 1 + LEN_LONGEST_STATE;
    let mut buf = vec![0u8; len];
    buf.fill(b'-');
    let divider = std::str::from_utf8(&buf[..]).expect("cannot fail");

    for (i, repo) in repos.into_iter().enumerate() {
        t.reset()?;
        if i != 0 {
            writeln!(t, "{}", divider)?;
        }
        for branch in repo.branches {
            t.fg(color::WHITE)?;
            let color = if branch.is_head {
                write!(t, "* ")?;
                COLOR_A
            } else {
                write!(t, "  ")?;
                COLOR_B
            };
            t.fg(color)?;
            write!(t, "{:<3} ", repo.name)?;
            t.fg(color::WHITE)?;
            write!(t, "|")?;
            t.fg(color)?;
            write!(t, " {}", branch.name)?;
            for _ in 0..len_longest_name - branch.name.len() {
                write!(t, " ")?;
            }
            if branch.is_head {
                write!(t, " {}", repo.state.as_str())?;
            }
            writeln!(t)?;
        }
    }
    t.reset()?;
    Ok(())
}

trait RepositoryStateExt {
    fn as_str(&self) -> &'static str;
}

impl RepositoryStateExt for RepositoryState {
    #[rustfmt::skip]
    fn as_str(&self) -> &'static str {
        use RepositoryState::*;
        match self {
            Clean                => "                  clean",
            Merge                => "                  merge",
            Revert               => "                 revert",
            RevertSequence       => "        revert-sequence",
            CherryPick           => "            cherry-pick",
            CherryPickSequence   => "    chery-pick-sequence",
            Bisect               => "                 bisect",
            Rebase               => "                 rebase",
            RebaseInteractive    => "     rebase-interactive",
            RebaseMerge          => "           rebase-merge",
            ApplyMailbox         => "          apply-mailbox",
            ApplyMailboxOrRebase => "apply-mailbox-or-rebase",
        }
    }
}
