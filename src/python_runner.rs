use std::io;
use std::io::Write;
use std::process::Command;
use crate::TestCase;

const DOCKER_WORKING_DIRECTORY: &'static str = "/usr/src/myapp";

pub(crate) struct PythonRunner {
    file_name: &'static str,
}

impl PythonRunner {
    pub(crate) fn new(file_name: &'static str) -> PythonRunner {
        PythonRunner { file_name }
    }

    pub(crate) fn run(&self, test_case: &TestCase) -> String {
        let output = Command::new("docker")
            .arg("run")
            .args(["-v", &*volume_mapping()])
            .args(["-w", DOCKER_WORKING_DIRECTORY])
            .args(["python:3"])
            .args(["python", self.file_name, &*test_case.num_courses, &*test_case.prerequisites])
            .output()
            .unwrap();

        // Log stdout and stderr
        // io::stdout().write_all(&output.stdout).unwrap();
        // io::stderr().write_all(&output.stderr).unwrap();

        let output_message = if output.status.success() {
            output.stdout
        } else {
            output.stderr
        };

        // Remove docker working directory if present
        String::from_utf8(output_message)
            .unwrap()
            .split(DOCKER_WORKING_DIRECTORY)
            .collect::<String>()
    }
}

fn volume_mapping() -> String {
    let path = std::env::current_dir().unwrap();
    format!("{}:{}", path.display(), DOCKER_WORKING_DIRECTORY)
}