use super::args::*;
use super::{ArgList, DescriptionMod, Filter, Modification, Report};
use crate::usage;
use nom::{branch::alt, combinator::*, sequence::*, IResult};
use taskchampion::Status;

// IMPLEMENTATION NOTE:
//
// For each variant of Subcommand, there is a private, empty type of the same name with a `parse`
// method and a `get_usage` method.  The parse methods may handle several subcommands, but always
// produce the variant of the same name as the type.
//
// This organization helps to gather the parsing and usage information into
// comprehensible chunks of code, to ensure that everything is documented.

/// A subcommand is the specific operation that the CLI should execute.
#[derive(Debug, PartialEq)]
pub(crate) enum Subcommand {
    /// Display the tool version
    Version,

    /// Display the help output
    Help {
        /// Give the summary help (fitting on a few lines)
        summary: bool,
    },

    /// Add a new task
    Add {
        modification: Modification,
    },

    /// Modify existing tasks
    Modify {
        filter: Filter,
        modification: Modification,
    },

    /// Lists (reports)
    List {
        report: Report,
    },

    /// Per-task information (typically one task)
    Info {
        filter: Filter,
        debug: bool,
    },

    /// Basic operations without args
    Gc,
    Sync,
}

impl Subcommand {
    pub(super) fn parse(input: ArgList) -> IResult<ArgList, Subcommand> {
        alt((
            Version::parse,
            Help::parse,
            Add::parse,
            Modify::parse,
            List::parse,
            Info::parse,
            Gc::parse,
            Sync::parse,
        ))(input)
    }

    pub(super) fn get_usage(u: &mut usage::Usage) {
        Version::get_usage(u);
        Help::get_usage(u);
        Add::get_usage(u);
        Modify::get_usage(u);
        List::get_usage(u);
        Info::get_usage(u);
        Gc::get_usage(u);
        Sync::get_usage(u);
    }
}

struct Version;

impl Version {
    fn parse(input: ArgList) -> IResult<ArgList, Subcommand> {
        fn to_subcommand(_: &str) -> Result<Subcommand, ()> {
            Ok(Subcommand::Version)
        }
        map_res(
            alt((
                arg_matching(literal("version")),
                arg_matching(literal("--version")),
            )),
            to_subcommand,
        )(input)
    }

    fn get_usage(u: &mut usage::Usage) {
        u.subcommands.push(usage::Subcommand {
            name: "version",
            syntax: "version",
            summary: "Show the TaskChampion version",
            description: "Show the version of the TaskChampion binary",
        });
    }
}

struct Help;

impl Help {
    fn parse(input: ArgList) -> IResult<ArgList, Subcommand> {
        fn to_subcommand(input: &str) -> Result<Subcommand, ()> {
            Ok(Subcommand::Help {
                summary: input == "-h",
            })
        }
        map_res(
            alt((
                arg_matching(literal("help")),
                arg_matching(literal("--help")),
                arg_matching(literal("-h")),
            )),
            to_subcommand,
        )(input)
    }

    fn get_usage(_u: &mut usage::Usage) {}
}

struct Add;

impl Add {
    fn parse(input: ArgList) -> IResult<ArgList, Subcommand> {
        fn to_subcommand(input: (&str, Modification)) -> Result<Subcommand, ()> {
            Ok(Subcommand::Add {
                modification: input.1,
            })
        }
        map_res(
            pair(arg_matching(literal("add")), Modification::parse),
            to_subcommand,
        )(input)
    }

    fn get_usage(u: &mut usage::Usage) {
        u.subcommands.push(usage::Subcommand {
            name: "add",
            syntax: "add [modification]",
            summary: "Add a new task",
            description: "
                Add a new, pending task to the list of tasks.  The modification must include a
                description.",
        });
    }
}

struct Modify;

impl Modify {
    fn parse(input: ArgList) -> IResult<ArgList, Subcommand> {
        fn to_subcommand(input: (Filter, &str, Modification)) -> Result<Subcommand, ()> {
            let filter = input.0;
            let mut modification = input.2;

            match input.1 {
                "prepend" => {
                    if let DescriptionMod::Set(s) = modification.description {
                        modification.description = DescriptionMod::Prepend(s)
                    }
                }
                "append" => {
                    if let DescriptionMod::Set(s) = modification.description {
                        modification.description = DescriptionMod::Append(s)
                    }
                }
                "start" => modification.active = Some(true),
                "stop" => modification.active = Some(false),
                "done" => modification.status = Some(Status::Completed),
                _ => {}
            }

            Ok(Subcommand::Modify {
                filter,
                modification,
            })
        }
        map_res(
            tuple((
                Filter::parse,
                alt((
                    arg_matching(literal("modify")),
                    arg_matching(literal("prepend")),
                    arg_matching(literal("append")),
                    arg_matching(literal("start")),
                    arg_matching(literal("stop")),
                    arg_matching(literal("done")),
                )),
                Modification::parse,
            )),
            to_subcommand,
        )(input)
    }

    fn get_usage(u: &mut usage::Usage) {
        u.subcommands.push(usage::Subcommand {
            name: "modify",
            syntax: "[filter] modify [modification]",
            summary: "Modify tasks",
            description: "
                Modify all tasks matching the filter.",
        });
        u.subcommands.push(usage::Subcommand {
            name: "prepend",
            syntax: "[filter] prepend [modification]",
            summary: "Prepend task description",
            description: "
                Modify all tasks matching the filter by inserting the given description before each
                task's description.",
        });
        u.subcommands.push(usage::Subcommand {
            name: "append",
            syntax: "[filter] append [modification]",
            summary: "Append task description",
            description: "
                Modify all tasks matching the filter by adding the given description to the end
                of each task's description.",
        });
        u.subcommands.push(usage::Subcommand {
            name: "start",
            syntax: "[filter] start [modification]",
            summary: "Start tasks",
            description: "
                Start all tasks matching the filter, additionally applying any given modifications."
        });
        u.subcommands.push(usage::Subcommand {
            name: "stop",
            syntax: "[filter] stop [modification]",
            summary: "Stop tasks",
            description: "
                Stop all tasks matching the filter, additionally applying any given modifications.",
        });
        u.subcommands.push(usage::Subcommand {
            name: "done",
            syntax: "[filter] done [modification]",
            summary: "Mark tasks as completed",
            description: "
                Mark all tasks matching the filter as completed, additionally applying any given
                modifications.",
        });
    }
}

struct List;

impl List {
    fn parse(input: ArgList) -> IResult<ArgList, Subcommand> {
        fn to_subcommand(input: (Report, &str)) -> Result<Subcommand, ()> {
            Ok(Subcommand::List { report: input.0 })
        }
        map_res(
            pair(Report::parse, arg_matching(literal("list"))),
            to_subcommand,
        )(input)
    }

    fn get_usage(u: &mut usage::Usage) {
        u.subcommands.push(usage::Subcommand {
            name: "list",
            syntax: "[filter] list",
            summary: "List tasks",
            description: "
                Show a list of the tasks matching the filter",
        });
    }
}

struct Info;

impl Info {
    fn parse(input: ArgList) -> IResult<ArgList, Subcommand> {
        fn to_subcommand(input: (Filter, &str)) -> Result<Subcommand, ()> {
            let debug = input.1 == "debug";
            Ok(Subcommand::Info {
                filter: input.0,
                debug,
            })
        }
        map_res(
            pair(
                Filter::parse,
                alt((
                    arg_matching(literal("info")),
                    arg_matching(literal("debug")),
                )),
            ),
            to_subcommand,
        )(input)
    }

    fn get_usage(u: &mut usage::Usage) {
        u.subcommands.push(usage::Subcommand {
            name: "info",
            syntax: "[filter] info",
            summary: "Show tasks",
            description: " Show information about all tasks matching the fiter.",
        });
        u.subcommands.push(usage::Subcommand {
            name: "debug",
            syntax: "[filter] debug",
            summary: "Show task debug details",
            description: " Show all key/value properties of the tasks matching the fiter.",
        });
    }
}

struct Gc;

impl Gc {
    fn parse(input: ArgList) -> IResult<ArgList, Subcommand> {
        fn to_subcommand(_: &str) -> Result<Subcommand, ()> {
            Ok(Subcommand::Gc)
        }
        map_res(arg_matching(literal("gc")), to_subcommand)(input)
    }

    fn get_usage(u: &mut usage::Usage) {
        u.subcommands.push(usage::Subcommand {
            name: "gc",
            syntax: "gc",
            summary: "Perform 'garbage collection'",
            description: "
                Perform 'garbage collection'.  This refreshes the list of pending tasks
                and their short id's.",
        });
    }
}

struct Sync;

impl Sync {
    fn parse(input: ArgList) -> IResult<ArgList, Subcommand> {
        fn to_subcommand(_: &str) -> Result<Subcommand, ()> {
            Ok(Subcommand::Sync)
        }
        map_res(arg_matching(literal("sync")), to_subcommand)(input)
    }

    fn get_usage(u: &mut usage::Usage) {
        u.subcommands.push(usage::Subcommand {
            name: "sync",
            syntax: "sync",
            summary: "Synchronize this replica",
            description: "
                Synchronize this replica locally or against a remote server, as configured.

                Synchronization is a critical part of maintaining the task database, and should
                be done regularly, even if only locally.  It is typically run in a crontask.",
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::argparse::Universe;

    const EMPTY: Vec<&str> = vec![];

    #[test]
    fn test_version() {
        assert_eq!(
            Subcommand::parse(argv!["version"]).unwrap(),
            (&EMPTY[..], Subcommand::Version)
        );
    }

    #[test]
    fn test_dd_version() {
        assert_eq!(
            Subcommand::parse(argv!["--version"]).unwrap(),
            (&EMPTY[..], Subcommand::Version)
        );
    }

    #[test]
    fn test_d_h() {
        assert_eq!(
            Subcommand::parse(argv!["-h"]).unwrap(),
            (&EMPTY[..], Subcommand::Help { summary: true })
        );
    }

    #[test]
    fn test_help() {
        assert_eq!(
            Subcommand::parse(argv!["help"]).unwrap(),
            (&EMPTY[..], Subcommand::Help { summary: false })
        );
    }

    #[test]
    fn test_dd_help() {
        assert_eq!(
            Subcommand::parse(argv!["--help"]).unwrap(),
            (&EMPTY[..], Subcommand::Help { summary: false })
        );
    }

    #[test]
    fn test_add_description() {
        let subcommand = Subcommand::Add {
            modification: Modification {
                description: DescriptionMod::Set(s!("foo")),
                ..Default::default()
            },
        };
        assert_eq!(
            Subcommand::parse(argv!["add", "foo"]).unwrap(),
            (&EMPTY[..], subcommand)
        );
    }

    #[test]
    fn test_add_description_multi() {
        let subcommand = Subcommand::Add {
            modification: Modification {
                description: DescriptionMod::Set(s!("foo bar")),
                ..Default::default()
            },
        };
        assert_eq!(
            Subcommand::parse(argv!["add", "foo", "bar"]).unwrap(),
            (&EMPTY[..], subcommand)
        );
    }

    #[test]
    fn test_modify_description_multi() {
        let subcommand = Subcommand::Modify {
            filter: Filter {
                universe: Universe::for_ids(vec![123]),
                ..Default::default()
            },
            modification: Modification {
                description: DescriptionMod::Set(s!("foo bar")),
                ..Default::default()
            },
        };
        assert_eq!(
            Subcommand::parse(argv!["123", "modify", "foo", "bar"]).unwrap(),
            (&EMPTY[..], subcommand)
        );
    }

    #[test]
    fn test_append() {
        let subcommand = Subcommand::Modify {
            filter: Filter {
                universe: Universe::for_ids(vec![123]),
                ..Default::default()
            },
            modification: Modification {
                description: DescriptionMod::Append(s!("foo bar")),
                ..Default::default()
            },
        };
        assert_eq!(
            Subcommand::parse(argv!["123", "append", "foo", "bar"]).unwrap(),
            (&EMPTY[..], subcommand)
        );
    }

    #[test]
    fn test_prepend() {
        let subcommand = Subcommand::Modify {
            filter: Filter {
                universe: Universe::for_ids(vec![123]),
                ..Default::default()
            },
            modification: Modification {
                description: DescriptionMod::Prepend(s!("foo bar")),
                ..Default::default()
            },
        };
        assert_eq!(
            Subcommand::parse(argv!["123", "prepend", "foo", "bar"]).unwrap(),
            (&EMPTY[..], subcommand)
        );
    }

    #[test]
    fn test_done() {
        let subcommand = Subcommand::Modify {
            filter: Filter {
                universe: Universe::for_ids(vec![123]),
                ..Default::default()
            },
            modification: Modification {
                status: Some(Status::Completed),
                ..Default::default()
            },
        };
        assert_eq!(
            Subcommand::parse(argv!["123", "done"]).unwrap(),
            (&EMPTY[..], subcommand)
        );
    }

    #[test]
    fn test_done_modify() {
        let subcommand = Subcommand::Modify {
            filter: Filter {
                universe: Universe::for_ids(vec![123]),
                ..Default::default()
            },
            modification: Modification {
                description: DescriptionMod::Set(s!("now-finished")),
                status: Some(Status::Completed),
                ..Default::default()
            },
        };
        assert_eq!(
            Subcommand::parse(argv!["123", "done", "now-finished"]).unwrap(),
            (&EMPTY[..], subcommand)
        );
    }

    #[test]
    fn test_start() {
        let subcommand = Subcommand::Modify {
            filter: Filter {
                universe: Universe::for_ids(vec![123]),
                ..Default::default()
            },
            modification: Modification {
                active: Some(true),
                ..Default::default()
            },
        };
        assert_eq!(
            Subcommand::parse(argv!["123", "start"]).unwrap(),
            (&EMPTY[..], subcommand)
        );
    }

    #[test]
    fn test_start_modify() {
        let subcommand = Subcommand::Modify {
            filter: Filter {
                universe: Universe::for_ids(vec![123]),
                ..Default::default()
            },
            modification: Modification {
                active: Some(true),
                description: DescriptionMod::Set(s!("mod")),
                ..Default::default()
            },
        };
        assert_eq!(
            Subcommand::parse(argv!["123", "start", "mod"]).unwrap(),
            (&EMPTY[..], subcommand)
        );
    }

    #[test]
    fn test_stop() {
        let subcommand = Subcommand::Modify {
            filter: Filter {
                universe: Universe::for_ids(vec![123]),
                ..Default::default()
            },
            modification: Modification {
                active: Some(false),
                ..Default::default()
            },
        };
        assert_eq!(
            Subcommand::parse(argv!["123", "stop"]).unwrap(),
            (&EMPTY[..], subcommand)
        );
    }

    #[test]
    fn test_stop_modify() {
        let subcommand = Subcommand::Modify {
            filter: Filter {
                universe: Universe::for_ids(vec![123]),
                ..Default::default()
            },
            modification: Modification {
                description: DescriptionMod::Set(s!("mod")),
                active: Some(false),
                ..Default::default()
            },
        };
        assert_eq!(
            Subcommand::parse(argv!["123", "stop", "mod"]).unwrap(),
            (&EMPTY[..], subcommand)
        );
    }

    #[test]
    fn test_list() {
        let subcommand = Subcommand::List {
            report: Report {
                ..Default::default()
            },
        };
        assert_eq!(
            Subcommand::parse(argv!["list"]).unwrap(),
            (&EMPTY[..], subcommand)
        );
    }

    #[test]
    fn test_list_filter() {
        let subcommand = Subcommand::List {
            report: Report {
                filter: Filter {
                    universe: Universe::for_ids(vec![12, 13]),
                    ..Default::default()
                },
            },
        };
        assert_eq!(
            Subcommand::parse(argv!["12,13", "list"]).unwrap(),
            (&EMPTY[..], subcommand)
        );
    }

    #[test]
    fn test_info_filter() {
        let subcommand = Subcommand::Info {
            debug: false,
            filter: Filter {
                universe: Universe::for_ids(vec![12, 13]),
                ..Default::default()
            },
        };
        assert_eq!(
            Subcommand::parse(argv!["12,13", "info"]).unwrap(),
            (&EMPTY[..], subcommand)
        );
    }

    #[test]
    fn test_debug_filter() {
        let subcommand = Subcommand::Info {
            debug: true,
            filter: Filter {
                universe: Universe::for_ids(vec![12]),
                ..Default::default()
            },
        };
        assert_eq!(
            Subcommand::parse(argv!["12", "debug"]).unwrap(),
            (&EMPTY[..], subcommand)
        );
    }

    #[test]
    fn test_gc() {
        let subcommand = Subcommand::Gc;
        assert_eq!(
            Subcommand::parse(argv!["gc"]).unwrap(),
            (&EMPTY[..], subcommand)
        );
    }

    #[test]
    fn test_gc_extra_args() {
        let subcommand = Subcommand::Gc;
        assert_eq!(
            Subcommand::parse(argv!["gc", "foo"]).unwrap(),
            (&vec!["foo"][..], subcommand)
        );
    }

    #[test]
    fn test_sync() {
        let subcommand = Subcommand::Sync;
        assert_eq!(
            Subcommand::parse(argv!["sync"]).unwrap(),
            (&EMPTY[..], subcommand)
        );
    }
}
