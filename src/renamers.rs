use std::error::Error;

use crate::{
    args::Args,
    entries::Entry,
    utils::{split___, split_eq},
};

struct RenameParam<'a> {
    p1: &'a str,
    t1: &'a str,
    p2: &'a str,
    t2: &'a str,
}

impl<'a> RenameParam<'a> {
    pub fn build(s: &str) -> Result<RenameParam, Box<dyn Error>> {
        let (lhs, rhs) = split_eq(&s)?;
        let (p1, t1) = split___(&lhs);
        let (p2, t2) = split___(&rhs);
        Ok(RenameParam { p1, p2, t1, t2 })
    }
}

pub struct Renames<'a> {
    r: Vec<RenameParam<'a>>,
}

impl<'a> Renames<'a> {
    pub fn build(args: &Args) -> Result<Renames, Box<dyn Error>> {
        let mut r = Vec::with_capacity(args.rename.len());

        for s in &args.rename {
            r.push(RenameParam::build(&s)?);
        }

        Ok(Renames { r })
    }
}

impl<'a> Renames<'a> {
    pub fn predicate_rename(&self, e: Entry) -> Entry {
        for r in &self.r {
            if e.project == r.p1 && e.task == r.t1 {
                return Entry {
                    project: r.p2.to_string(),
                    task: r.t2.to_string(),
                    ..e
                };
            }
        }
        e
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renameparam() {
        let RenameParam { p1, t1, p2, t2 } =
            RenameParam::build("Project1___Task1=Project2___Task2").unwrap();
        assert_eq!(p1, "Project1");
        assert_eq!(t1, "Task1");
        assert_eq!(p2, "Project2");
        assert_eq!(t2, "Task2");
    }

    #[test]
    fn renameparam_empty_task() {
        let RenameParam { p1, t1, p2, t2 } = RenameParam::build("Project1___=Project2___").unwrap();
        assert_eq!(p1, "Project1");
        assert_eq!(t1, "");
        assert_eq!(p2, "Project2");
        assert_eq!(t2, "");
    }

    #[test]
    fn renameparam_project_only() {
        let RenameParam { p1, t1, p2, t2 } = RenameParam::build("Project1=Project2").unwrap();
        assert_eq!(p1, "Project1");
        assert_eq!(t1, "");
        assert_eq!(p2, "Project2");
        assert_eq!(t2, "");
    }
}
