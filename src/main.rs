#![allow(dead_code)]
#![allow(unused_imports)]

use std::io::{self, BufRead, Write,  BufReader, BufWriter};
use std::fs::{self, File};
use std::path::Path;
use std::os::unix::fs::PermissionsExt;
use std::os::linux::fs::MetadataExt;


fn visit_dirs(dir: &Path, size: &mut u64) -> io::Result<()> {
    if dir.is_dir() {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries {
                let entry = entry.expect("expected file");
                let path = entry.path();

                if path.is_dir() {
                    let metadata = fs::symlink_metadata(&path).expect("could not read metadata from folder");
                    *size += metadata.len();
                    if metadata.permissions().readonly() {
                        println!("Don't have permission to access '{}'", path.display());
                    } else if metadata.file_type().is_symlink() {
                        //println!("This is a symbolic link '{}'", path.display());
                    } else {
                        visit_dirs(&path, size)?;
                    }

                } else if path.is_file() {

                    match fs::metadata(&path) {
                        Ok(metadata) => *size += metadata.len(),
                        Err(_) => println!("could not read metadata from file {}  ble", path.display()),
                    }
                }
            }
        }
    }

    Ok(())
}

fn main() -> std::io::Result<()> {
    let args: Vec<String> = std::env::args().collect();

    let ble = if args.len() < 2 {
        vec![".".to_string()]
    } else {
        let (_, nodes) = args.split_at(1);
        nodes.to_vec()
    };


    let mut sizes = ble.iter().map(|str_path|{
        let path = Path::new(str_path);
        let mut size: u64 = 0;
        let metadata = fs::metadata(&path).expect("could not read metadata from file");
        if !path.is_dir() {
            size = metadata.len();
        } else {
            size += metadata.len();
            visit_dirs(path, &mut size).expect("Failed top level");
        }

        (str_path, size)
    })
    .collect::<Vec<_>>();

    sizes.sort_by(|a, b| a.1.cmp(&b.1));

    for (path, size) in sizes {
        let s = size as f64;

        let ending = match s.log(1024_f64) as u64 {
            4 => format!("{:.2}{}",s/1024_f64.powi(4),"T"),
            3 => format!("{:.2}{}",s/1024_f64.powi(3),"G"),
            2 => format!("{:.2}{}",s/1024_f64.powi(2),"M"),
            1 => format!("{:.2}{}",s/1024_f64,"K"),
            _ => s.to_string()
        };
        println!("{}\t{}", ending, path);
    }

    Ok(())
}

