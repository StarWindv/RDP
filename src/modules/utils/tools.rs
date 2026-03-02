pub struct Tools {}


impl Tools {
    pub const fn build_version() -> &'static str {
        concat!(
            env!("CARGO_PKG_DESCRIPTION"), "\n",
            "Version : ", env!("CARGO_PKG_VERSION"), "\n",
            "License : ", env!("CARGO_PKG_LICENSE"), "\n",
            "Homepage: ", env!("CARGO_PKG_HOMEPAGE"), "\n"
        )
    }

    pub const fn build_usage() -> &'static str {
        concat!(
            env!("CARGO_PKG_DESCRIPTION"),
            "\n",
            "Unix Usage:\n",
            " -c:  Execute String Command\n",
            " -s:  Force stdin as Command Source\n",
            " -i:  Force Into Interactive Mode\n",
            " - : Option Terminator (Stop Parsing Options)\n\n",
            "RDP Usage:\n",
            "              -h/--help : Output This Message and Exit\n",
            "      -ri/--run-ir <ir> : Execute Optimized Binary Format IR\n",
            "     -do/--debug-output : Output Every Debug Output to Stdout\n",
            " -di/--dump-ir <source> : Optimize the Source and Save it's IR in Binary Format\n",
            "-hi/--human-ir <source> : Optimize the Source and Save it's IR in Human-Readable Format\n",
        )
    }
}
