use libactionkv::ActionKV;

#[cfg(target_os = "windows")]
const USAGE: &str = "
Usage:
    akv_mem.exe FILE get KEY
    akv_mem.exe FILE delete KEY
    akv_mem.exe FILE insert KEY VALUE
    akv_mem.exe FILE update KEY VALUE
";

#[cfg(not(target_os = "windows"))]
const USAGE: &str = "
Usage:
    akv_mem FILE get KEY
    akv_mem FILE delete KEY
    akv_mem FILE insert KEY VALUE
    akv_mem FILE update KEY VALUE
";

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let filename = args.get(1).expect(&USAGE);
    let action = args.get(2).expect(&USAGE).as_ref();
    let key = args.get(3).expect(&USAGE).as_ref();
    let value = args.get(4).expect(&USAGE).as_ref();

    let path = std::path::Path::new(&filename);
    let mut akv = ActionKV::open(path).expect("Failed to open ActionKV");
    akv.load().expect("Failed to load ActionKV");

    match action {
        "get" => {
            match akv.get(key).unwrap() {
                Some(value) => println!("{}", String::from_utf8(value).unwrap()),
                None => eprintln!("Key not found"),
            }
        }
        "delete" => {
            akv.delete(key).unwrap();
        }
        "insert" => {
            let val = value.expect(&USAGE).as_ref();
            akv.insert(key, val).unwrap();
        }
        "update" => {
            let val = value.expect(&USAGE).as_ref();
            akv.update(key, val).unwrap();
        }
        _ => eprintln!("Unknown action"),
    }
}