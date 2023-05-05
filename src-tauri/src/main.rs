// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

struct NotionInfo {
    key: String,
    _database_id_string: String,
    database_id: DatabaseId,
}

impl NotionInfo {
    fn new() -> Self {
        let Ok(key) = env::var("NOTION_KEY") else {
            panic!("NOTION_KEY not provided!");
        };

        let Ok(database_id_string) = env::var("NOTION_DATABASE_ID") else {
            panic!("NOTION_DATABASE_ID not provided!");
        };

        let database_id = DatabaseId::from_str(&database_id_string).unwrap();

        Self {
            key,
            _database_id_string: database_id_string,
            database_id,
        }
    }
}

use std::str::FromStr;

#[derive(serde::Serialize)]
struct TaskResponse {
    task: String,
}

#[derive(Debug, Clone, Serialize)]
struct NotionError {
    message: String,
}

impl From<notion::Error> for NotionError {
    fn from(value: notion::Error) -> Self {
        dbg!(&value);
        Self {
            message: value.to_string(),
        }
    }
}

impl From<&str> for NotionError {
    fn from(value: &str) -> Self {
        Self {
            message: value.into(),
        }
    }
}

use notion::models::{properties::PropertyValue, search::StatusCondition, text::RichText};
use tauri::{Manager, PhysicalSize, Position, WindowBuilder};

#[tauri::command]
fn show_window(window: tauri::Window) {
    window.show().unwrap();
}

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
async fn get_in_progress_item(
    notion_api: tauri::State<'_, notion::NotionApi>,
    notion_info: tauri::State<'_, NotionInfo>,
) -> Result<TaskResponse, NotionError> {
    let database_query = DatabaseQuery {
        sorts: None,
        paging: None,
        filter: Some(FilterCondition::Property {
            property: "Status".to_string(),
            condition: PropertyCondition::Status(StatusCondition::Equals(String::from(
                "In progress",
            ))),
        }),
    };
    let res = notion_api
        .query_database(&notion_info.database_id, database_query)
        .await?;

    let item = res
        .results
        .first()
        .ok_or("Unable to find an in progress status")?;

    // Yes this query below looks disgusting, I agree
    let task = match item
        .properties
        .properties
        .get("Name")
        .ok_or("Unable to find a Name on struct")?
    {
        PropertyValue::Title { title, .. } => match title.first().ok_or("No title for item!")? {
            RichText::Text { text, .. } => text.content.clone(),
            _ => unreachable!(),
        },
        _ => unreachable!(),
    };

    Ok(TaskResponse { task })
}

use std::env;

use notion::{
    ids::DatabaseId,
    models::search::{DatabaseQuery, FilterCondition, PropertyCondition},
};
use serde::Serialize;

const WIDTH_RATIO: f64 = 33f64 / 100f64;
const HEIGHT_RATIO: f64 = 1f64 / 15f64;

fn main() {
    dotenv::dotenv().ok();

    let notion_info = NotionInfo::new();
    let notion_api = notion::NotionApi::new(notion_info.key.clone()).unwrap();

    tauri::Builder::default()
        .manage(notion_info)
        .manage(notion_api)
        .invoke_handler(tauri::generate_handler![get_in_progress_item, show_window])
        .setup(|app| {
            let w = app.get_window("main").unwrap();
            
            let display = w.current_monitor()?.unwrap();
            let (display_width, display_height) = (display.size().width, display.size().height);

            let (width, height) = (
                (display_width as f64 * WIDTH_RATIO) as u32,
                (display_height as f64 * HEIGHT_RATIO) as u32,
            );

            w.set_size(PhysicalSize::new(width, height))?;

            w.set_position(Position::Physical(tauri::PhysicalPosition {
                // display_width / 2 - 2 * width
                x: (display_width as f64 / 2f64 - width as f64 / 2f64) as i32,
                // display_height - height
                y: (display_height - height) as i32,
            }))?;

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
