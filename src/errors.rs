error_chain! {

    foreign_links {
        Fmt(::std::fmt::Error);
        Io(::std::io::Error);
        SerdeYaml(::serde_yaml::Error);
        SetLog(::log::SetLoggerError);
    }

    errors {
        InvalidAccountName(path: String) {
            description("invalid account name")
            display("invalid account name: '{}'", path)
        }
        AlreadyExists(path: String) {
            description("an account already exists at that path")
            display("an account already exists at the path {}", path)
        }
        InvalidDeposit(path: String, amount: String) {
            description("invalid deposit, the account is a derived account")
            display("invalid deposit of {} to {}, {1} is a derived account", amount, path)
        }
        UnwrapNode
    }
}
