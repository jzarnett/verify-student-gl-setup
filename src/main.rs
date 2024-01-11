use std::{env, fs};
use std::fs::File;
use std::io::{BufRead, BufReader, Lines, Write};

use gitlab::api::Query;
use gitlab::api::users::UsersBuilder;
use gitlab::Gitlab;
use serde::Deserialize;

const UW_GITLAB_URL: &str = "git.uwaterloo.ca";

#[derive(Debug, Deserialize)]
struct ProjectUser {
    id: u64,
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 6 {
        println!(
            "Usage: {} <list_of_students.csv> <token_file>",
            args.first().unwrap()
        );
        println!(
            "Example: {} students.csv token.git",
            args.first().unwrap()
        );
        return;
    }
    let token = read_token_file(args.get(5).unwrap());

    let students = parse_csv_file(args.get(4).unwrap());
    let client = Gitlab::new(String::from(UW_GITLAB_URL), token).unwrap();
    verify_students(client, students)
}

fn verify_students(client: Gitlab, students: Vec<String>) {
    let mut found_students = Vec::new();
    let mut not_found_students = Vec::new();

    for student in students {
        println!("Looking up student {student}...");
        let gl_user_id = retrieve_user_id(&client, &student);
        if let Some(id) = gl_user_id {
            println!("Student {student} has a user ID of {id}.");
            found_students.push(student)
        } else {
            not_found_students.push(student)
        }
    }

    let mut found_file = File::create("found.txt").unwrap();
    for f in found_students {
        let line = format!("{f}\n");
        found_file.write_all(line.as_bytes()).unwrap();
    }
    let mut not_found_file = File::create("not_found.txt").unwrap();
    for f in not_found_students {
        let line = format!("{f}\n");
        not_found_file.write_all(line.as_bytes()).unwrap()
    }
}

fn retrieve_user_id(client: &Gitlab, student: &String) -> Option<u64> {
    let gl_user_builder = UsersBuilder::default().username(student).build().unwrap();
    let gl_user: Vec<ProjectUser> = gl_user_builder.query(client).unwrap();
    return if gl_user.is_empty() {
        None
    } else {
        Option::from(gl_user.first().unwrap().id)
    };
}

fn parse_csv_file(filename: &String) -> Vec<String> {
    let mut result: Vec<String> = Vec::new();
    let lines = read_lines(filename);

    for line in lines {
        let line = line.unwrap();
        result.push(String::from(line.trim()));
    }
    result
}

fn read_lines(filename: &String) -> Lines<BufReader<File>> {
    let file = File::open(filename).unwrap();
    BufReader::new(file).lines()
}

fn read_token_file(filename: &String) -> String {
    let mut token = fs::read_to_string(filename)
        .unwrap_or_else(|_| panic!("Unable to read token from file {filename}"));
    token.retain(|c| !c.is_whitespace());
    token
}
