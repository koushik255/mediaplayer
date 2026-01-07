use super::app_types::SubtitleEntry;
use ass_parser::{AssFile, Dialogue};
use srtlib::{Subtitles, Timestamp};
use std::time::Duration;

pub fn ts_to_duration(t: &Timestamp) -> Duration {
    let (h, m, s, ms) = t.get();
    Duration::from_millis(
        (h as u64) * 3_600_000 + (m as u64) * 60_000 + (s as u64) * 1_000 + (ms as u64),
    )
}

pub fn ass_time_to_duration(t: &str) -> Option<Duration> {
    let mut parts = t.split(':');
    let h = parts.next()?.parse::<u64>().ok()?;
    let m = parts.next()?.parse::<u64>().ok()?;
    let sec_cs = parts.next()?;

    let mut sc = sec_cs.split('.');
    let s = sc.next()?.parse::<u64>().ok()?;
    let cs = sc.next()?.parse::<u64>().ok()?;

    let millis = h * 3_600_000 + m * 60_000 + s * 1_000 + cs * 10;
    Some(Duration::from_millis(millis))
}

pub fn strip_ass_tags(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut in_brace = false;
    for c in s.chars() {
        match c {
            '{' => in_brace = true,
            '}' => in_brace = false,
            _ if !in_brace => out.push(c),
            _ => {}
        }
    }
    out.replace("\\N", "\n").trim().to_string()
}

pub fn parse_example_subs(file: &str) -> Result<Vec<SubtitleEntry>, String> {
    let mut entries: Vec<SubtitleEntry> = Vec::new();
    println!("Before file parse from file");

    if let Ok(subs) = Subtitles::parse_from_file(file, None) {
        let mut v = subs.to_vec();
        v.sort();
        for s in v {
            entries.push(SubtitleEntry {
                start: ts_to_duration(&s.start_time),
                end: ts_to_duration(&s.end_time),
                text: s.text.trim().to_string(),
            });
        }
    }

    if let Ok(ass_file) = AssFile::from_file(file) {
        let dialogues: Vec<Dialogue> = ass_file.events.get_dialogues();
        for d in dialogues {
            if let (Some(b), Some(e), Some(txt)) = (d.get_start(), d.get_end(), d.get_text()) {
                println!("before duration ass parse");
                println!("{}", file);
                let (start, end) = (
                    ass_time_to_duration(&b).unwrap(),
                    ass_time_to_duration(&e).unwrap(),
                );
                {
                    let clean = strip_ass_tags(&txt);
                    println!("subtitles before push {:?}", &txt.clone());
                    entries.push(SubtitleEntry {
                        start,
                        end,
                        text: clean,
                    });
                }
            }
        }
    }

    entries.sort_by_key(|e| e.start);

    println!("LOADED SUBTITLE FILE");
    Ok(entries)
}
