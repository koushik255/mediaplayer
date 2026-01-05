use super::app_types::Dbchoose;
use rusqlite::{Connection, Error, Result as SqliteResult, params};

pub fn db(time: f64, vid_file: String, subfile: String) {
    let conn = Connection::open("mydb.sqlite3").expect("error connecting to db");
    println!(
        "DB: params which are being inserted {} {} {}",
        time, vid_file, subfile
    );

    conn.execute(
        "CREATE TABLE IF NOT EXISTS last (
                    time  REAL,
                    file  TEXT NOT NULL,
                    subfile  TEXT NOT NULL
)",
        [],
    )
    .expect("erroring creating db table");
    conn.execute("DELETE from last", [])
        .expect("Error deleting lsat table");

    conn.execute(
        "INSERT INTO last (time,file,subfile) VALUES (?1,?2,?3)",
        params![time, vid_file, subfile],
    )
    .expect("erroing inserting last");

    println!("succesfully added last to db");
}

pub fn save_settings(
    subtitle_offset: f64,
    subtitle_offset_vertical: f64,
    subtitle_offset_horizontal: f64,
    video_width: f32,
    video_height: f32,
    volume: f64,
) {
    let conn = Connection::open("mydb.sqlite3").expect("error connecting to db");
    println!("Saving settings to db");

    conn.execute(
        "CREATE TABLE IF NOT EXISTS settings (
                    subtitle_offset REAL,
                    subtitle_offset_vertical REAL,
                    subtitle_offset_horizontal REAL,
                    video_width REAL,
                    video_height REAL,
                    volume REAL
)",
        [],
    )
    .expect("error creating settings table");
    conn.execute("DELETE from settings", [])
        .expect("Error deleting settings table");

    conn.execute(
        "INSERT INTO settings (subtitle_offset, subtitle_offset_vertical, subtitle_offset_horizontal, video_width, video_height, volume) VALUES (?1,?2,?3,?4,?5,?6)",
        params![
            subtitle_offset,
            subtitle_offset_vertical,
            subtitle_offset_horizontal,
            video_width,
            video_height,
            volume
        ],
    )
    .expect("error inserting settings");

    println!("successfully saved settings");
}

pub fn load_settings() -> Result<(f64, f64, f64, f32, f32, f64), String> {
    let conn =
        Connection::open("mydb.sqlite3").map_err(|e| format!("Error connecting to db: {}", e))?;

    let mut stmt = conn
        .prepare("SELECT subtitle_offset, subtitle_offset_vertical, subtitle_offset_horizontal, video_width, video_height, volume FROM settings")
        .map_err(|e| format!("Statement error {}", e))?;

    match stmt.query_row([], |row| {
        Ok((
            row.get(0)?,
            row.get(1)?,
            row.get(2)?,
            row.get(3)?,
            row.get(4)?,
            row.get(5)?,
        ))
    }) {
        Ok(settings) => {
            println!("Loaded settings: {:?}", settings);
            Ok(settings)
        }
        Err(Error::QueryReturnedNoRows) => {
            println!("No settings found, using defaults");
            Err("No settings found".to_string())
        }
        Err(e) => Err(format!("Database error {}", e)),
    }
}

pub fn db_for_each(time: f64, vid_file: String, subfile: String) {
    let conn = Connection::open("allvids.sqlite3").expect("Error connecting to db");
    println!("db for each now");

    conn.execute(
        "CREATE TABLE IF NOT EXISTS all (
                    time REAL,
                    file TEXT NOT NULL UNIQUE,
                    subfile TEXT NOT NULL UNIQUE
)",
        [],
    )
    .expect("error creaing db table");

    conn.execute(
        "INSERT INTO all(time,file,subfile) VALUES(?1,?2,?3)",
        params![time, vid_file, subfile],
    )
    .expect("Erroring inserting last");

    println!("after inserting to everything")
}

pub fn db_get_all() -> SqliteResult<Vec<Dbchoose>> {
    let conn = Connection::open("allvids.sqlite3").expect("errpr connecting ot db");
    println!("db get all");

    let mut stmt = conn
        .prepare("SELECT time,file,subfile FROM all")
        .expect("error selcting db all");

    let all = stmt
        .query_map([], |row| {
            Ok(Dbchoose {
                time: row.get(0).expect("error time db all"),
                vid_file: row.get(1).expect("error file db all"),
                subfile: row.get(2).expect("errrpr sub db"),
            })
        })
        .expect("error getting query ")
        .collect::<Result<Vec<_>, _>>()
        .expect("error collect");
    Ok(all)
}

pub fn db_get_last() -> Result<Dbchoose, String> {
    let conn =
        Connection::open("mydb.sqlite3").map_err(|e| format!("Erorr connecting to db: {}", e))?;

    let mut stmt = conn
        .prepare("SELECT time,file,subfile FROM last")
        .map_err(|e| format!("Statement error {}", e))?;

    match stmt.query_row([], |row| {
        Ok(Dbchoose {
            time: row.get(0)?,
            vid_file: row.get(1)?,
            subfile: row.get(2)?,
        })
    }) {
        Ok(last) => {
            println!("{:?}", last);
            Ok(last)
        }
        Err(Error::QueryReturnedNoRows) => {
            println!("Could not find last");
            Err("Could not find last".to_string())
        }
        Err(e) => Err(format!("Database error {}", e)),
    }
}
