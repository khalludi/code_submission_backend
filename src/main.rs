use std::collections::HashMap;
use std::fmt::Display;
use axum::{
    routing::{get, post},
    http::StatusCode,
    Json, Router,
};
use serde::{de, Deserialize, Deserializer, Serialize};
use std::process::Command;
use std::fs::File;
use std::io;
use std::io::Write;
use std::str::FromStr;
use serde::de::DeserializeOwned;

#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt::init();

    // build our application with a route
    let app = Router::new()
        .route("/", get(root))
        .route("/runCode", post(run_code));

    // run our app with hyper, listening globally on port 5000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:5000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

// basic handler that responds with a static string
async fn root() -> &'static str {
    "Hello, World!"
}

async fn run_code(Json(payload): Json<CreateCode>) -> (StatusCode, Json<Output>) {
    // println!("{:?}", payload);

    let input_filename = "test.py";
    let mut file = File::create(input_filename).expect("unable to create file");
    file.write_all(payload.code.as_bytes()).expect("could not write code to file");

    let path = std::env::current_dir().unwrap();
    let docker_working_directory = "/usr/src/myapp";
    let volume_mapping = format!("{}:{}", path.display(), docker_working_directory);

    let mut results_map: HashMap<String, String> = HashMap::new();
    for (key, test_case) in payload.test_cases.into_iter() {
        let grader_output = call_grader(&test_case);
        results_map.insert(key, String::from_utf8(grader_output).unwrap());
    }
    println!("{:?}", results_map);

    let output = Command::new("docker")
        .arg("run")
        .args(["-v", volume_mapping.as_str()])
        .args(["-w", docker_working_directory])
        .args(["python:3", "python", input_filename])
        .output()
        .unwrap();

    io::stdout().write_all(&output.stdout).unwrap();
    io::stderr().write_all(&output.stderr).unwrap();

    let output_message = if output.status.success() {
                                      output.stdout
                                  } else {
                                      output.stderr
                                  };

    // Remove docker working directory if present
    let filtered_output = String::from_utf8(output_message)
        .unwrap()
        .split(docker_working_directory)
        .collect::<String>();

    let result = Output {
        output: filtered_output,
        grader_output: results_map
    };

    (StatusCode::OK, Json(result))
}

fn call_grader(test_case: &TestCase) -> Vec<u8> {
    let path = std::env::current_dir().unwrap();
    let docker_working_directory = "/usr/src/myapp";
    let volume_mapping = format!("{}:{}", path.display(), docker_working_directory);

    let output = Command::new("docker")
        .arg("run")
        .args(["-v", volume_mapping.as_str()])
        .args(["-w", docker_working_directory])
        .args(["python:3", "python", "grader.py", test_case.num_courses.as_str(), test_case.prerequisites.as_str()])
        .output()
        .unwrap();

    io::stdout().write_all(&output.stdout).unwrap();
    io::stderr().write_all(&output.stderr).unwrap();

    let output_message = if output.status.success() {
        output.stdout
    } else {
        output.stderr
    };

    return output_message;
}

#[derive(Deserialize, Debug)]
struct CreateCode {
    code: String,
    #[serde(rename = "testCaseHash")]
    test_cases: HashMap<String, TestCase>
}

#[derive(Deserialize, Debug)]
struct TestCase {
    #[serde(rename = "numCourses")]
    num_courses: String,
    prerequisites: String,
}

#[derive(Serialize, Debug)]
struct Output {
    output: String,
    #[serde(rename = "graderOutput")]
    grader_output: HashMap<String, String>,
}