use std::process::Command;

pub fn command_to_string(command: &Command) -> String {
    let mut result = String::new();

    // Add the program
    result.push_str(command.get_program().to_str().unwrap());

    // Add the arguments
    for arg in command.get_args() {
        if let Some(arg_str) = arg.to_str() {
            result.push(' ');
            // Check if the argument needs to be quoted
            if arg_str.contains(char::is_whitespace) {
                result.push('"');
                result.push_str(arg_str.replace('"', "\\\"").as_str());
                result.push('"');
            } else {
                result.push_str(arg_str);
            }
        }
    }

    result
}
