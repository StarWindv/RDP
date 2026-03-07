use crate::modules::utils::rich::rich::Rich;
use clap::{Arg, ArgAction, ArgMatches, Command};
/**
 * Fuck `argh`
 * <br>argh's macro receive static str/string description msg only
 * <br>so I must use `concat` macro, but my lib cannot provide macro function (so difficult!)
 *
 * Fuck `bpaf`
 * <br>wise guy
 * <br>auto remove version line break
 * <br>not support long-short args like `-ri`
 * <br>not support single delimiter arg `args1 - args2`
 *
 * And you, Fuck `clap`
 * <br>wise guy.
 * <br>not support long-short args like `-ri`
 * <br>not support single delimiter arg `args1 - args2`
 *
 * I compromised.
 * <br>I can't create arg single `-` in my cli like another old unix cli.
 * <br>I plead guilty.
 *
 * I gave in, changed arg `-` to `--`, which is all I can do and the library supports it was born that way.
 * <br>I changed `-ri` to `R`
 * <br>I changed `-do` to `d`
 * <br>I changed `-di` to `D`
 * <br>I changed `-hi` to `H`
 * <br>I'm a sinner, I'm betraying proj's dev-guide.
 *
 * All of you are not as good as `argparse` is
*/

#[derive(Debug)]
pub struct Cli;


impl Cli {
    pub fn build_version() -> String {
        let version: &str = concat!(
            "\n<LastStart><Bold><Underline>[255,255,255]Version[/] : ",
            /*
            Puzzling Action!
            When I use `SetTrue` and `disable_version_flag` to build my custom version message,
            the clap library will automatically add the blank line before my version info,
            so I must use ANSI codes fallback to last line's start,
            then my custom version info will be displayed correctly.
            */
            /*
            Also, you can try this tag to recover default style:
            ```
            <Reset>
            ```
            The behaviors of `[/]` and `<Reset>` are equivalent.
            */
            env!("CARGO_PKG_VERSION"),
            "\n<Bold><Underline>[255,255,255]License[/] : ",
            env!("CARGO_PKG_LICENSE"),
            "\n<Bold><Underline>[255,255,255]Homepage[/]: ",
            env!("CARGO_PKG_HOMEPAGE"),
        );
        Rich::process(version)
        /*
        Thanks to `disable_version_flag`!
        If I still use the clap's default version flag,
        then I must use this:
        ```
        Box::leak(Rich::process(version).into_boxed_str())
        ```
        Because clap forces you to use `long_version` method,
        and `long_version` receive a `&str`;
        If I use `&*` to unpack a String, then I will defended by the lifetime checker
        */
    }

    pub fn build_usage() -> String {
        let usage: String = format!(
            "<Bold><Underline>[255,255,255]{} v{} [{}]\n\n\
                Unix Usage:<Reset>\n\
                [255,255,255]\
                -c :  Execute String Command\n\
                -s :  Force stdin as Command Source\n\
                -i :  Force Into Interactive Mode\n\
                -- :  Option Terminator (Stop Parsing Options)\n\n\
                <Bold><Underline>[255,255,255]\
                RDP Usage:<Reset>\n\
                <Bold>[255,255,255]-h/--help<Reset>               : Output This Message and Exit\n\
                <Bold>[255,255,255]-v/--version<Reset>            : Output Project Version and Exit\n\
                <Bold>[255,255,255]-d/--debug-output<Reset>      : Output Every Debug Output to Stdout\n\
                <Bold>[255,255,255]-R/--run-ir <ir-source><Reset> : Execute Optimized Binary Format IR Code\n\
                <Bold>[255,255,255]-D/--dump-ir   <source><Reset> : Optimize the Source and Save it's IR in Binary Format\n\
                <Bold>[255,255,255]-H/--human-ir  <source><Reset> : Optimize the Source and Save it's IR in Human-Readable Format",
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_VERSION"),
            env!("BUILD_TIME")
        );
        Rich::process(&*usage)
    }

    pub fn parse() -> clap::ArgMatches {
        Command::new(env!("CARGO_PKG_NAME"))
            .help_template(Self::build_usage())
            .disable_help_flag(true)
            .disable_help_subcommand(true)
            .disable_version_flag(true)
            .arg(
                Arg::new("help")
                    .short('h')
                    .long("help")
                    .help("Output This Message and Exit")
                    .action(ArgAction::Help),
            )
            .arg(
                Arg::new("version")
                    .short('v')
                    .long("version")
                    .help("Output Project Version Info and Exit")
                    .action(ArgAction::SetTrue),
            )
            .arg(
                Arg::new("execute_command")
                    .short('c')
                    .help("Execute String Command")
                    .action(ArgAction::Set)
                    .num_args(1)
                    .value_name("COMMAND"),
            )
            .arg(
                Arg::new("force_stdin")
                    .short('s')
                    .help("Force stdin as Command Source")
                    .action(ArgAction::SetTrue),
            )
            .arg(
                Arg::new("force_interactive")
                    .short('i')
                    .help("Force Into Interactive Mode")
                    .action(ArgAction::SetTrue),
            )
            .arg(
                Arg::new("run_ir")
                    .short('R')
                    .long("run-ir")
                    .help("Execute Optimized Binary Format IR")
                    .action(ArgAction::Set)
                    .num_args(1)
                    .value_name("IR"),
            )
            .arg(
                Arg::new("debug_output")
                    .short('d')
                    .long("debug-output")
                    .help("Output Every Debug Output to Stdout")
                    .action(ArgAction::SetTrue),
            )
            .arg(
                Arg::new("dump_ir")
                    .short('D')
                    .long("dump-ir")
                    .help("Optimize the Source and Save it's IR in Binary Format")
                    .action(ArgAction::Set)
                    .num_args(1)
                    .value_name("SOURCE"),
            )
            .arg(
                Arg::new("human_ir")
                    .short('H')
                    .long("human-ir")
                    .help("Optimize the Source and Save it's IR in Human-Readable Format")
                    .action(ArgAction::Set)
                    .num_args(1)
                    .value_name("SOURCE"),
            )
            .get_matches()
    }

    pub fn run() -> ArgMatches{
        let argv = Self::parse();
        if argv.get_flag("version") {
            print!("{}", Self::build_version());
        }
        argv
    }
}
