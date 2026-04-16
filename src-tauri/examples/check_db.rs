// Component #1 test: verify SQLite tables contain expected data.
// Usage: cargo run --example check_db -- "<path to search.db>"

use rusqlite::{Connection, OpenFlags};
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: cargo run --example check_db -- <path to search.db>");
        std::process::exit(1);
    }
    let db_path = &args[1];
    println!("[SV#1] Opening read-only: {}\n", db_path);

    let conn = match Connection::open_with_flags(db_path, OpenFlags::SQLITE_OPEN_READ_ONLY) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("[SV#1] FAIL: cannot open DB: {}", e);
            std::process::exit(2);
        }
    };

    // 1) List all tables
    println!("[SV#1] === All tables in DB ===");
    if let Ok(mut stmt) =
        conn.prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")
    {
        let rows = stmt.query_map([], |r| r.get::<_, String>(0));
        if let Ok(it) = rows {
            for name in it.flatten() {
                let count: Result<i64, _> =
                    conn.query_row(&format!("SELECT COUNT(*) FROM \"{}\"", name), [], |r| r.get(0));
                match count {
                    Ok(n) => println!("  {:<30} rows={}", name, n),
                    Err(_) => println!("  {:<30} (count failed)", name),
                }
            }
        }
    }

    // 2) Full schema of note_links
    println!("\n[SV#1] === note_links schema ===");
    if let Ok(mut stmt) = conn.prepare("PRAGMA table_info(note_links)") {
        let rs = stmt.query_map([], |r| {
            Ok((
                r.get::<_, i64>(0).unwrap_or(0),
                r.get::<_, String>(1).unwrap_or_default(),
                r.get::<_, String>(2).unwrap_or_default(),
                r.get::<_, i64>(3).unwrap_or(0),
            ))
        });
        if let Ok(it) = rs {
            for r in it.flatten() {
                println!(
                    "  col#{} name={:?} type={:?} notnull={}",
                    r.0, r.1, r.2, r.3
                );
            }
        }
    }

    // 3) target_path emptiness: populated vs empty
    println!("\n[SV#1] === note_links target_path populated vs empty ===");
    let populated: Result<i64, _> = conn.query_row(
        "SELECT COUNT(*) FROM note_links WHERE target_path IS NOT NULL AND target_path <> ''",
        [],
        |r| r.get(0),
    );
    let empty: Result<i64, _> = conn.query_row(
        "SELECT COUNT(*) FROM note_links WHERE target_path IS NULL OR target_path = ''",
        [],
        |r| r.get(0),
    );
    println!("  populated target_path: {}", populated.unwrap_or(-1));
    println!("  empty target_path:     {}", empty.unwrap_or(-1));

    // 4) Sample 3 rows WHERE target_path is populated
    println!("\n[SV#1] === Sample 3 note_links WHERE target_path <> '' ===");
    if let Ok(mut stmt) = conn.prepare(
        "SELECT source_path, target_path, link_type, status FROM note_links \
         WHERE target_path IS NOT NULL AND target_path <> '' LIMIT 3",
    ) {
        let rs = stmt.query_map([], |r| {
            Ok((
                r.get::<_, String>(0).unwrap_or_default(),
                r.get::<_, String>(1).unwrap_or_default(),
                r.get::<_, String>(2).unwrap_or_default(),
                r.get::<_, String>(3).unwrap_or_default(),
            ))
        });
        if let Ok(it) = rs {
            let mut any = false;
            for r in it.flatten() {
                any = true;
                println!(
                    "  src={:?}\n    tgt={:?}\n    type={:?} status={:?}\n",
                    r.0, r.1, r.2, r.3
                );
            }
            if !any {
                println!("  (no rows returned — every single row has empty target_path)");
            }
        }
    }

    // 5) Sample rows with EMPTY target_path — show all their columns (JSON blob maybe?)
    println!("\n[SV#1] === Sample 3 note_links WHERE target_path = '' (full columns) ===");
    if let Ok(mut stmt) = conn.prepare("SELECT * FROM note_links LIMIT 3") {
        // Discover column count and names first
        let names: Vec<String> = stmt
            .column_names()
            .into_iter()
            .map(|s| s.to_string())
            .collect();
        println!("  columns: {:?}", names);

        let rs = stmt.query_map([], |row| {
            let mut vals = Vec::new();
            for i in 0..names.len() {
                let v: rusqlite::types::Value = row.get(i).unwrap_or(rusqlite::types::Value::Null);
                vals.push(format!("{:?}", v));
            }
            Ok(vals)
        });
        if let Ok(it) = rs {
            for (idx, r) in it.flatten().enumerate() {
                println!("  row {}:", idx);
                for (i, v) in r.iter().enumerate() {
                    // Char-boundary-safe truncate
                    let s = if v.chars().count() > 80 {
                        let truncated: String = v.chars().take(80).collect();
                        format!("{}...(truncated)", truncated)
                    } else {
                        v.clone()
                    };
                    println!("    {} = {}", names.get(i).map(|s| s.as_str()).unwrap_or("?"), s);
                }
            }
        }
    }

    // 6b) Distribution of target_name — how many distinct values, sample top 10 by frequency
    println!("\n[SV#1] === note_links target_name distribution (top 10 by frequency) ===");
    if let Ok(mut stmt) = conn.prepare(
        "SELECT target_name, COUNT(*) FROM note_links \
         WHERE target_name IS NOT NULL AND target_name <> '' \
         GROUP BY target_name ORDER BY 2 DESC LIMIT 10",
    ) {
        let rs = stmt.query_map([], |r| {
            Ok((
                r.get::<_, String>(0).unwrap_or_default(),
                r.get::<_, i64>(1).unwrap_or(0),
            ))
        });
        if let Ok(it) = rs {
            for r in it.flatten() {
                let n: String = r.0.chars().take(60).collect();
                println!("  count={:>6} target_name={:?}", r.1, n);
            }
        }
    }
    // Also: total distinct target_name values
    let distinct: Result<i64, _> = conn.query_row(
        "SELECT COUNT(DISTINCT target_name) FROM note_links \
         WHERE target_name IS NOT NULL AND target_name <> ''",
        [],
        |r| r.get(0),
    );
    println!("  distinct target_name values: {}", distinct.unwrap_or(-1));

    // How many target_names resolve to an existing note (by basename)?
    let resolvable: Result<i64, _> = conn.query_row(
        "SELECT COUNT(*) FROM note_links l \
         WHERE EXISTS (SELECT 1 FROM note_meta n WHERE n.name = l.target_name)",
        [],
        |r| r.get(0),
    );
    println!("  links whose target_name matches a note_meta.name: {}", resolvable.unwrap_or(-1));

    // 6) Are there any tag-related tables?
    println!("\n[SV#1] === Tables with 'tag' in the name ===");
    if let Ok(mut stmt) = conn.prepare(
        "SELECT name FROM sqlite_master WHERE type='table' AND name LIKE '%tag%'",
    ) {
        let rows = stmt.query_map([], |r| r.get::<_, String>(0));
        let mut found = false;
        if let Ok(it) = rows {
            for name in it.flatten() {
                found = true;
                let count: Result<i64, _> =
                    conn.query_row(&format!("SELECT COUNT(*) FROM \"{}\"", name), [], |r| r.get(0));
                println!("  {} rows={}", name, count.unwrap_or(-1));
            }
        }
        if !found {
            println!("  (no tables match '%tag%')");
        }
    }

    println!("\n[SV#1] DONE");
}
