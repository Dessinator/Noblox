use std::fmt::Debug;
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use rbx_dom_weak::WeakDom;
use rbx_xml::DecodeOptions;
use rfd::FileDialog;

const ALLOWED_EXTENSIONS: [&'static str; 4] = ["rbxl", "rbxlx", "rbxm", "rbxmx"];

// Note:    .rbxl(x?) -> world
//          .rbxm(x?) -> model/object
//          .rbx[lm]x -> xml
//          .rbx[lm]  -> binary

fn main() -> Result<(), String> {
    println!("--------------------------------------------------");
    println!("Hello! Please input your Roblox Place/Model file path.");
    println!("--------------------------------------------------");

    // let file_path: PathBuf = get_file_path_from_terminal();
    let file_path: Box<Path> = FileDialog::new()
        .add_filter("Roblox places/models", &ALLOWED_EXTENSIONS)
        .pick_file()
        .ok_or_else(|| "No file selected".to_owned())? // return error code 1
        .into();

    println!("--------------------------------------------------");
    println!("Constructing DOM...");
    println!("--------------------------------------------------");

    let dom: WeakDom = get_dom_from_extension(file_path.as_ref())?;

    println!("--------------------------------------------------");
    println!("DOM successfully constructed.");
    println!("Root instances in file:");
    println!("--------------------------------------------------");

    debug_print_dom_instance(&dom, dom.root(), 0);

    Ok(())
}

/// Reads stdin until a newline character and parses the input into the given type.
/// Returns the requested type T if successful, or a String with a reason why the error occurred
/// in the case of an error.
fn read_stdin<T: FromStr>() -> Result<T, String>
where
    <T as FromStr>::Err: Debug,
{
    // create objects to read text from console while there is no graphical ui
    let stdin = std::io::stdin();
    let mut line = String::new();
    let input: T;

    // reads a single u8 from the terminal
    match stdin.read_line(&mut line) {
        Ok(_) => {
            line = String::from(line.trim());
            input = match line.parse::<T>() {
                Ok(parsed) => parsed,
                Err(_) => return Err(String::from("Could not parse the input into type T.")),
            };
            return Ok(input);
        }
        Err(error) => println!("error: {error}"),
    };

    Err(String::from("Could not parse the input into type T."))
}

/// Prompts the terminal to accept a valid file path from the user. Will loop if any errors occur.
/// Checks if the file at the given path exists, and ensures that the file at the given path is
/// saved in .rbxl, .rbxlx, .rbxm, or .rbxmx format before returning a Path.
fn get_file_path_from_terminal() -> PathBuf {
    loop {
        println!();
        println!();

        // get the file path of the roblox file from the terminal.
        let input_file_path_string: String;
        input_file_path_string = match read_stdin::<String>() {
            Ok(input) => String::from(input.trim()),
            Err(_) => {
                println!("Could not parse your input. Please try again.");
                continue;
            }
        };

        let input_file_path: &Path = Path::new(&input_file_path_string);

        // if the file path does not exist, retry entry.
        match input_file_path.try_exists() {
            Ok(exists) => {
                if !exists {
                    println!("--------------------------------------------------");
                    println!(
                        "That isn't a valid file. Please ensure you are selecting a file saved in rbxl, rbxlx, rbxm, or rbxmx format."
                    );
                    println!("--------------------------------------------------");
                    continue;
                }
            }
            Err(why) => {
                println!("--------------------------------------------------");
                println!("An error occurred. ({}) \nPlease try again.", why);
                println!("--------------------------------------------------");
                continue;
            }
        }

        let display = input_file_path.display();
        let extension = input_file_path.extension().unwrap().to_str().unwrap();

        println!("--------------------------------------------------");

        // if the given file path does not point to a valid roblox file (.rbxl, .rbxm, .rbxlx, .rbxmx)
        // then restart.
        match extension {
            // Roblox Place file saved in binary format.
            "rbxl" => {
                println!("You entered: \n{} \n(This is a .rbxl file)", display);
                println!("--------------------------------------------------");
                return input_file_path.to_path_buf();
            }
            // Roblox Place file saved in xml format.
            "rbxlx" => {
                println!("You entered: \n{} \n(This is a .rbxlx file)", display);
                println!("--------------------------------------------------");
                return input_file_path.to_path_buf();
            }
            // Roblox Model file saved in binary format.
            "rbxm" => {
                println!("You entered: \n{} \n(This is a .rbxm file)", display);
                println!("--------------------------------------------------");
                return input_file_path.to_path_buf();
            }
            // Roblox Model file saved in xml format.
            "rbxmx" => {
                println!("You entered: \n{} \n(This is a .rbxmx file)", display);
                println!("--------------------------------------------------");
                return input_file_path.to_path_buf();
            }
            _ => {
                println!(
                    "That isn't a valid Roblox file. Please ensure you are selecting a file saved in rbxl, rbxlx, rbxm, or rbxmx format."
                );
                println!("--------------------------------------------------");
                continue;
            }
        }
    }
}

/// Reads the file extension of the given Path and returns a WeakDom using the method
/// corresponding to it. Returns a WeakDom representation of the file at the end of the Path
/// if successful, or a String with a reason why the error occurred in the case of an error.
fn get_dom_from_extension(path_buffer: &Path) -> Result<WeakDom, String> {
    let extension: &str = path_buffer.extension().unwrap().to_str().unwrap();
    let display = path_buffer.display();
    let error_message = String::from(format!(
        "An error occurred while retrieving a DOM from the given file path: {}",
        display
    ));

    // get the DOM from the given file path. Panics if there are any errors since there shouldn't
    // be any at this point.
    match extension {
        // Roblox Place file saved in binary format.
        "rbxl" | "rbxm" => get_dom_from_binary(&path_buffer).map_err(|_| error_message),
        // Roblox Model file saved in xml format.
        "rbxlx" | "rbxmx" => get_dom_from_xml(&path_buffer).map_err(|_| error_message),
        _ => Err(error_message),
    }
}

/// Reads the file in binary format at the end of the Path. Returns a WeakDom representation of
/// the file at the end of the Path if successful, or a DecodeError in the case of an error.
fn get_dom_from_binary(file_path: &Path) -> Result<WeakDom, rbx_binary::DecodeError> {
    let buffer = BufReader::new(File::open(file_path).unwrap());
    let result = rbx_binary::from_reader(buffer);
    return result;
}

/// Reads the file in xml format at the end of the Path. Returns a WeakDom representation of
/// the file at the end of the Path if successful, or a DecodeError in the case of an error.
fn get_dom_from_xml(file_path: &Path) -> Result<WeakDom, rbx_xml::DecodeError> {
    let buffer = BufReader::new(File::open(file_path).unwrap());
    let result = rbx_xml::from_reader(buffer, DecodeOptions::new());
    return result;
}

fn debug_print_dom_instance(dom: &WeakDom, inst: &rbx_dom_weak::Instance, depth: usize) {
    for c_ref in inst.children() {
        let child = dom.get_by_ref(*c_ref).unwrap();
        println!("{}- {}: {}", "\t".repeat(depth), child.name, child.class);
        debug_print_dom_instance(dom, child, depth + 1);
    }
}