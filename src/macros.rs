#[macro_export]
macro_rules! cmd {
    ($fmt:literal $(, $($args:expr),*)? ) => {{
        // Replace format brackets with values in scope of the caller
        let formatted_cmd = format!($fmt $(, $($args),*)?);
        println!("PHIBAN spawn: {}", formatted_cmd);

        let mut iter = formatted_cmd.split_whitespace();
        let program = iter.next().expect("No first item?");
        let args = iter;

        // Execute and inherit stdout/stderr
        let status = std::process::Command::new(program)
            .args(args)
            .status()
            .expect("Failed to launch command");

        // Check success; panic (or handle differently) on failure
        if !status.success() {
            dbg!(status);
            panic!("We don't handle this failure yet!");
        }
    }};
}
