use std::{hash, path, str};
use std::fs::File;
use std::fs::OpenOptions;
use std::io::{BufRead, BufReader};
use std::io::BufWriter;
use std::io::Write;
use std::path::Path;
use std::thread;
use std::time::Duration;

use clokwerk::{Scheduler, TimeUnits};
use clokwerk::Interval::*;
use notify_rust::Notification;
use scraper::{Html, Selector};
use sha1::{Digest, Sha1};
use dotenv::dotenv;
use std::process::exit;

fn main() {
    let error_message = "A .env file with a SECRETO_URL field is required for this program to function properly.";

    let secreto_url = dotenv::var("SECRETO_URL")
        .expect(error_message);

    if secreto_url.eq("") {
        println!("{}", error_message);
        exit(1)
    }

    let mut scheduler = Scheduler::new();
    let message_storage_path = "./messages.txt";

    fetch_secreto(secreto_url.as_str(), message_storage_path);

    scheduler.every(1.minutes()).run(move ||
        fetch_secreto(secreto_url.as_str(), message_storage_path)
    );

    loop {
        scheduler.run_pending();
        thread::sleep(Duration::from_millis(10));
    }
}

fn fetch_secreto(url: &str, stored_message_path: &str) {
    println!("{}", "Fetching from secreto...");

    if let Ok(response) = reqwest::blocking::get(url) {
        if let Ok(html_string) = response.text() {

            let document = Html::parse_document(&html_string);
            let message_selector = Selector::parse("div.msg_block").unwrap();

            let mut incoming_messages: Vec<String> = vec![];

            for message_element in document.select(&message_selector) {
                let message_text = message_element.text().collect::<Vec<_>>()
                    .join(" ");

                incoming_messages.push(message_text.trim().to_string());
            }

            let message_file_path = stored_message_path;
            let mut file_for_reading = OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .open(message_file_path)
                .unwrap();

            let mut existing_messages: Vec<String> = vec![];
            {
                let reader = BufReader::new(file_for_reading);
                let mut temp_string = String::new();

                for line in reader.lines() {
                    let unwrapped = line.unwrap();

                    if unwrapped.len() == 0 {
                        existing_messages.push(temp_string);
                        temp_string = String::new();
                    } else {
                        temp_string = format!("{}\n{}", temp_string, unwrapped)
                    }
                }
            }

            let mut new_messages: Vec<String> = vec![];
            for incoming_message in &incoming_messages {
                let mut found = false;
                'inner: for existing_message in &existing_messages {
                    if existing_message.trim().eq(incoming_message.trim()) {
                        found = true;
                        break 'inner;
                    }
                }
                if !found {
                    new_messages.push(incoming_message.clone())
                }
            }

            if new_messages.to_vec().len() > 0 {
                let message_body = new_messages.to_vec().join("\n\n");

                Notification::new()
                    .summary("New Secreto(s)!")
                    .body(message_body.as_str())
                    .summary(message_body.as_str())
                    .show()
                ;
            }

            let mut file_for_writing = OpenOptions::new()
                .write(true)
                .append(true)
                .create(true)
                .open(message_file_path)
                .unwrap();

            for new_message in new_messages.iter().rev() {
                file_for_writing.write_all(new_message.as_bytes());
                file_for_writing.write_all(b"\n\n");
            }
        } else {
            println!("Failed to parse response.")
        }
    } else {
        println!("Failed to fetch from {}", url)
    }
}