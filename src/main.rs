mod config;

use std::cmp::Ordering;

use git2::{BranchType, Repository};
use term::color::{self, Color};

use crate::config::Config;

const COLOR_A: Color = color::WHITE;
const COLOR_B: Color = color::BLUE;

struct Repo {
    name: String,
    branches: Vec<Branch>,
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
        repos.push(Repo { name, branches });
    }

    const COLUMNS: usize = 80;
    let mut buf = vec![0u8; COLUMNS];
    buf.fill(b'-');
    let divider = std::str::from_utf8(&buf[..]).expect("cannot fail");

    for (i, repo) in repos.into_iter().enumerate() {
        t.reset()?;
        if i != 0 {
            writeln!(t, "{}", divider)?;
        }
        for branch in repo.branches {
            if !branch.is_head && (branch.name == "master" || branch.name == "stable") {
                continue;
            }
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
            writeln!(t)?;
        }
    }
    t.reset()?;
    Ok(())
}
