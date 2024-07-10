mod python_runner;

use std::collections::HashMap;
use std::fmt::Display;
use axum::{
    routing::{get, post},
    http::StatusCode,
    Json, Router,
};
use serde::{de, Deserialize, Deserializer, Serialize};
use std::fs::File;
use std::io::{BufRead, BufReader, Read, Write};
use std::str::FromStr;
use serde::de::DeserializeOwned;
use crate::python_runner::PythonRunner;

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
    let input_filename = "test.py";
    let mut file = File::create(input_filename).expect("unable to create file");
    file.write_all(payload.code.as_bytes()).expect("could not write code to file");

    let user_addon_filename = "user_addon.py";
    let file2 = File::open(user_addon_filename).expect("could not open user addon file");
    let mut reader = BufReader::new(file2);

    for line in reader.lines().map(|l| l.unwrap()) {
        writeln!(file, "{}", line).unwrap();
    }

    let mut grader_map: HashMap<String, String> = HashMap::new();
    let mut user_map: HashMap<String, String> = HashMap::new();
    for (key, test_case) in payload.test_cases.into_iter() {
        let judge_runner = PythonRunner::new("grader.py");
        grader_map.insert(key.clone(), judge_runner.run(&test_case));

        let user_runner = PythonRunner::new(input_filename);
        user_map.insert(key, user_runner.run(&test_case));
    }
    println!("{:?}", user_map);
    println!("{:?}", grader_map);

    let result = Output {
        user_output: user_map,
        grader_output: grader_map
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
    prerequisites: String,
}

#[derive(Serialize, Debug)]
struct Output {
    #[serde(rename = "userOutput")]
    user_output: HashMap<String, String>,
    #[serde(rename = "graderOutput")]
    grader_output: HashMap<String, String>,
}