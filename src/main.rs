use std::collections::HashSet;
use std::env;
use std::fs;
use std::process::Command;

struct Config<'a> {
    mode: &'a str,
    backup_file: &'a str,
    restore_file: &'a str,
}

impl<'a> Config<'a> {
    fn new(mode: &'a str) -> Self {
        Self {
            mode,
            backup_file: "backup",
            restore_file: "backup",
        }
    }

    fn set_backup_file(&mut self, filename: &'a str) {
        self.backup_file = filename;
    }

    fn set_restore_file(&mut self, filename: &'a str) {
        self.restore_file = filename;
    }
}

fn main() {
    let input_args: Vec<String> = env::args().collect();

    let mut use_defaults = true;
    if input_args.len() < 2 {
        println!("Not enough arguments");
        std::process::exit(1);
    } else if input_args.len() >= 3 {
        use_defaults = false;
    }
    let mut config = create_config(input_args[1].as_str());
    match config.mode {
        "backup" => {
            if !use_defaults {
                config.set_backup_file(input_args[2].as_str());
            }
            println!("Backup Mode");
            create_backup(&config);
        }
        "restore" => {
            if !use_defaults {
                config.set_restore_file(input_args[2].as_str());
            }
            println!("Restore Mode");
            install_apps(&config);
        }
        &_ => {
            println!("Incorrect Parameters");
            std::process::exit(1);
        }
    }
}

fn create_config(mode: &str) -> Config {
    Config::new(mode)
}

fn create_backup(config: &Config) {
    let filename = "/var/log/apt/history.log";
    let queries = vec!["Commandline: apt install", "Commandline: apt-get install"];

    let mut results_hash = HashSet::new();

    let contents =
        fs::read_to_string(filename).expect("Something went wrong with reading the file");

    for line in contents.lines() {
        for query in queries.iter() {
            if line.contains(query) {
                let split_line: Vec<&str> = line.split_whitespace().collect();

                results_hash.insert(split_line.last().unwrap().to_string());
            }
        }
    }
    let results_string: Vec<String> = results_hash.into_iter().collect();
    fs::write(config.backup_file, results_string.join("\n")).expect("Couldn't write data.");
}

fn install_apps(config: &Config) {
    let contents = fs::read_to_string(config.restore_file).expect("Unable to open backup file");

    for line in contents.lines() {
        match install_app(line) {
            Ok(app_name) => println!("Installed: {}", app_name),
            Err((e, app_name)) => println!("Error in app {} : {}", app_name, e),
        }
    }
}

fn install_app(app_line: &str) -> Result<String, (String, String)> {
    let output = Command::new("apt-get")
        .args(["install", app_line])
        .output()
        .expect("Could not run installation.");

    let raw_err_string = String::from_utf8(output.stderr).unwrap();

    let split_err_string: Vec<&str> = raw_err_string.split("\n\n").collect();
    let app_name = app_line.to_string();

    if raw_err_string.contains("E:") {
        if raw_err_string.contains("WARNING:") {
            return Err((split_err_string[1].to_string(), app_name));
        }
        Err((split_err_string[0].to_string(), app_name))
    } else {
        Ok(app_name)
    }
}
