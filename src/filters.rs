use std::str::FromStr;

use crate::{args::Args, entries::Entry};

pub struct FilterParam {
    ignored: bool,
    billable: bool,
    ignore_list: Vec<String>,
}

impl FilterParam {
    pub fn build(args: &Args) -> FilterParam {
        FilterParam {
            ignored: args.ignored,
            billable: args.billable,
            ignore_list: args.ignore_list.clone(),
        }
    }
}

pub fn predicate_filter(e: &Entry, p: &FilterParam) -> bool {
    let mut cond = false;
    for i in &p.ignore_list {
        let s: Vec<&str> = i.split("___").collect();
        if s.len() == 1 {
            cond = cond || e.project == s[0];
        } else {
            cond = cond || (e.project == s[0] && e.task == s[1]);
        }
    }
    cond = !cond;
    if !p.billable {
        cond = cond && e.billable;
    }
    if !p.ignored {
        cond = cond
            && !e.tags.contains(
                String::from_str("Ignore")
                    .as_ref()
                    .expect("Hard coded strings should be valid"),
            );
    }
    cond
}
