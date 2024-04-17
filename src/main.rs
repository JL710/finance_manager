fn main() {
    let args = std::env::args().collect::<Vec<String>>();
    if args.len() != 2 {
        panic!("Please provide a command line argument to specify the mode of the application.");
    }

    match args[1].as_str() {
        "server" => fm_server::run(),
        "gui" => fm_gui::run(),
        _ => panic!("Invalid mode specified."),
    }
}
