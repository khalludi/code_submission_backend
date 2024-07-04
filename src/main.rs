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
    println!("{:?}", payload);

    let input_filename = "test.py";
    let mut file = File::create(input_filename).expect("unable to create file");
    file.write_all(payload.code.as_bytes()).expect("could not write code to file");

    let path = std::env::current_dir().unwrap();
    let docker_working_directory = "/usr/src/myapp";
    let volume_mapping = format!("{}:{}", path.display(), docker_working_directory);

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
    };

    (StatusCode::OK, Json(result))
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
    //#[serde(deserialize_with = "json_from_str")]
    prerequisites: String,
}

#[derive(Serialize, Debug)]
struct Output {
    output: String,
}

fn primitive_from_str<'de, T, D>(deserializer: D) -> Result<T, D::Error>
    where T: FromStr,
          T::Err: Display,
          D: Deserializer<'de>
{
    let s = String::deserialize(deserializer)?;
    T::from_str(&s).map_err(de::Error::custom)
}

fn json_from_str<'de, T, D>(deserializer: D) -> Result<T, D::Error>
    where T: DeserializeOwned,
          D: Deserializer<'de>
{
    let s = String::deserialize(deserializer)?;
    serde_json::from_str(&s).map_err(de::Error::custom)
}