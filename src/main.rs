// Consider replacing docopt with clap
extern crate docopt;
use docopt::Docopt;
#[macro_use]
extern crate serde_derive;

extern crate rusqlite;
use rusqlite::Connection;

extern crate rustyline;
use rustyline::Editor;

extern crate clipboard;
use clipboard::ClipboardProvider;
use clipboard::ClipboardContext;

use std::path::Path;

use std::io;
use std::io::prelude::*;

// TODO: check for keybase
// TODO: user agnostic path
const DATA_PATH: &'static str = "/keybase/private/ojensen/pw.dat";

const USAGE: &'static str = "
Command-line password manager using Keybase for cloud storage mechanism.
You must be logged in to Keybase.

Usage:
  pw new [<category>] <name>
  pw edit <name>
  pw delete <name>
  pw list [<category>]
  pw show <name>
  pw copy <name> (u|p)

Options:
  -h --help     Show this screen.
  --version     Show version.
";

#[derive(Debug, Deserialize)]
struct Args {
    cmd_new: bool,
    cmd_list: bool,
    cmd_show: bool,
    cmd_copy: bool,
    cmd_edit: bool,
    cmd_delete: bool,

    cmd_u: bool,
    cmd_p: bool,

    arg_name: String,
    arg_category: Option<String>
}

#[derive(Debug)]
struct Credential {
    id: u32,
    name: String,
    category: String,
    username: String,
    password: String
}

fn main() {

    let conn = initialize_datastore();

    let args: Args = Docopt::new(USAGE)
                            .and_then(|d| d.deserialize())
                            .unwrap_or_else(|e| e.exit());
    //println!("{:?}", args);

    if args.cmd_new {
        new_credential(&conn, args.arg_category, args.arg_name);
    }
    else if args.cmd_list {
        list_credentials(&conn, args.arg_category);
    }
    else if args.cmd_show {
        show_credential(&conn, args.arg_name);
    }
    else if args.cmd_copy {
        copy_credential(&conn, args.arg_name, args.cmd_u);
    }
    else if args.cmd_edit {
        // TODO: edit a given credential
        println!("Edit not yet implemented");
    }
    else if args.cmd_delete {
        // TODO: delete a given credential
        println!("Delete not yet implemented");
    }

}

fn new_credential(conn: &rusqlite::Connection, category: Option<String>, name: String) {
    let category = match category {
        Some(c) => c,
        None => "".to_string()
    };

    if name_exists(conn, &name) {
        println!("A credential with this name already exists.");
        return;
    }

    print!("Creating new credentials named \"{}\"", name);
    if category != "" {
        print!(" in category \"{}\"", category)
    }
    println!("");

    let mut rl = Editor::<()>::new();
    let username = rl.readline("Username: ").expect("No username supplied.");
    let password = rl.readline("Password: ").expect("No password supplied.");
    conn.execute("INSERT INTO credentials
        (name, category, username, password)
        values
        (?1, ?2, ?3, ?4)",
        &[&name, &category, &username, &password]
    ).unwrap();

    println!("Saved.");
}

fn list_credentials(conn: &rusqlite::Connection, category: Option<String>) {
    // TODO: how do we differentiate yes/no category in a non-stupid way?
    let mut statement = match category.to_owned() {
        Some(c) => conn.prepare("SELECT category, name FROM credentials WHERE category = ?1 ORDER BY category,name").unwrap(),
        None => conn.prepare("SELECT category, name FROM credentials ORDER BY category,name").unwrap()
    };
    let mut rows = match category.to_owned() {
        Some(c) => statement.query(&[&category]).unwrap(),
        None => statement.query(&[]).unwrap()
    };

    // TODO: how do we do this in a non-stupid way?
    let mut previousCategory = "".to_string();
    while let Some(result_row) = rows.next() {
        let row = result_row.unwrap();
        let category: String = row.get(0);
        let name: String = row.get(1);
        if previousCategory != category {
            println!("\nCategory: {}", category);
            previousCategory = category;
        }
        println!("    {}", name);
    }
}

fn show_credential(conn: &rusqlite::Connection, name: String) {
    let credential = get_credential(conn, name);
    println!("{}:\n    {}\n    {}", credential.name, credential.username, credential.password);
}

fn get_credential(conn: &rusqlite::Connection, name: String) -> Credential {
    conn.query_row("SELECT * FROM credentials WHERE name = ?1", &[&name], |row| {
        Credential {
            id: row.get(0),
            name: row.get(1),
            category: row.get(2),
            username: row.get(3),
            password: row.get(4)
        }
    }).expect("No such credential saved.")
}

fn copy_credential(conn: &rusqlite::Connection, name: String, username: bool) {
    let credential = get_credential(conn, name);
    let mut ctx: ClipboardContext = ClipboardProvider::new().unwrap();
    if username {
        ctx.set_contents(credential.username);
        println!("{} username copied to clipboard.", credential.name);
    } else {
        ctx.set_contents(credential.password);
        println!("{} password copied to clipboard.", credential.name);
    }
    pause("(press enter to clear)");
}

fn initialize_datastore() -> rusqlite::Connection {
    let db_exists = Path::new(DATA_PATH).is_file();
    let conn = Connection::open(DATA_PATH).unwrap();
    if !db_exists {
        conn.execute("CREATE TABLE credentials (
                      id              INTEGER PRIMARY KEY,
                      name            TEXT UNIQUE NOT NULL,
                      category        TEXT,
                      username        TEXT,
                      password        TEXT
                      )", &[]).unwrap();
    }
    return conn;
}


fn name_exists(conn: &rusqlite::Connection, name: &str) -> bool {
    match conn.query_row("SELECT count(*) FROM credentials WHERE name = ?1", &[&name], |row| {
            let val: i64 = row.get(0);
            val
        }) {
        Ok(0) => return false,
        _ => return true
    }
}

fn pause(message: &str) {
    let mut stdin = io::stdin();
    let mut stdout = io::stdout();

    print!("{}", message);
    stdout.flush().unwrap();

    let _ = stdin.read(&mut [0u8]).unwrap();
}