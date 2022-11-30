use fltk::{app, prelude::*, frame::*, group::*, button::*, window::*, output::MultilineOutput, tree::{Tree, TreeReason, TreeSort}};
use smash_arc::{ArcFile, Hash40, ArcLookup};

use std::sync::{Mutex, Arc};
use std::rc::Rc;

mod tree_utils;
mod file_info;

use tree_utils::{build_tree, get_path, extract_tree_item};

fn main() {
    // let param_label_path = fltk::dialog::file_chooser("Open ParamLabels.csv", "*.csv", ".", false).unwrap();
    // prc::hash40::set_custom_labels(
    //     prc::hash40::read_custom_labels(param_label_path)
    //         .unwrap()
    //         .into_iter()
    // );
    
    let app = app::App::default();
    let arc_path = fltk::dialog::file_chooser("Open data.arc", "*.arc", ".", false).unwrap();
    let label_path = fltk::dialog::file_chooser("Open Labels", "*", ".", false).unwrap();
    //let arc_path = "/home/jam/re/ult/900/data.arc";
    //let label_path = "/home/jam/dev/ult/smash-arc/hash_labels.txt";
    // let arc_path = "D:/Roms/Switch/Super Smash Bros. Ultimate Updates/10.0.0/romfs/data.arc";
    // let label_path = "C:/Modding/Super Smash Bros. Ultimate/data.arc dumps/13.0.1/hashes.txt";
    Hash40::set_global_labels_file(&label_path).unwrap();
    let arc = &*Box::leak(Box::new(ArcFile::open(arc_path).unwrap()));

    let mut wind = Window::default()
        .with_size(1000, 500)
        .center_screen()
        .with_label("DirInfo Tree Viewer");

    let mut pack = Pack::default().size_of(&wind).center_of(&wind);
    pack.set_type(PackType::Horizontal);
    pack.set_spacing(0);
    
    let mut tree_pack = Pack::default().with_size(500, 500);

    let mut tree = Tree::new(0, 0, 500, 475, "Tree");
    tree.set_root_label("/");
    tree.set_sort_order(TreeSort::Ascending);

    let dir_infos = &arc.get_dir_infos()
        .iter()
        .collect::<Vec<_>>();

    for dir in dir_infos {
        match dir.parent.global_label() {
            Some(_hash_to_str) => {},
            None => {
                let lbl = format!("{:#x}", dir.path.hash40().as_u64());
                let hash_to_str = dir.path.hash40().global_label().unwrap_or(lbl);
                println!("No parent for {}!", hash_to_str);
                
                if !hash_to_str.starts_with("0x") {
                    // Parent dir info
                    let new_paths = hash_to_str.split("/").collect::<Vec<&str>>();
                    if new_paths.len() >= 2 {
                        let mut full_path: String = String::new();
                        for i in 0..new_paths.len() {
                            full_path.push_str(new_paths[i]);
                            full_path.push('/');
                            match tree.add(&full_path) {
                                Some(mut res) => {
                                    res.set_label_color(fltk::enums::Color::DarkRed)
                                },
                                None => {}
                            }
                        }
                    } else {
                        tree.add(&hash_to_str).unwrap().set_label_color(fltk::enums::Color::DarkRed);
                    }

                    build_tree(&arc, &mut tree, &hash_to_str, 1).unwrap();
                }
            }
        }
    }

    
    tree_pack.add(&tree);

    let mut button_pack = Pack::default().with_size(500, 25);

    let mut extract_button = Button::default().with_size(50, 25).with_label("Extract");
    button_pack.add(&extract_button);

    tree_pack.add(&button_pack);
    tree_pack.end();
    pack.add(&tree_pack);

    let mut frame = Frame::default().with_size(500, 500).right_of(&tree, 0);
    frame.set_color(Color::Red);

    let output = Arc::new(MultilineOutput::default().center_of(&frame).size_of(&frame));

    tree.set_callback2(move |tree| {
        match tree.callback_reason() {
            TreeReason::Opened => {
                let /* mut */ path = get_path(tree.callback_item().unwrap());
                if let Err(_) = build_tree(arc, tree, &path, 1) {
                    build_tree(arc, tree, &path, 1).unwrap();
                }
            }
            TreeReason::Selected => {
                let path = get_path(tree.callback_item().unwrap());
                let output = Arc::clone(&output);
                std::thread::spawn(move || {
                    output.set_value(&file_info::get(arc, &path));
                });
            }
            _ => ()
        }
    });

    tree.get_items()
        .unwrap()
        .into_iter()
        .for_each(|mut node| node.close());
    tree.root().unwrap().open();


    let tree = Rc::new(Mutex::new(tree));
    let tree_ref = Rc::clone(&tree);
    
    extract_button.set_callback(move || {
        extract_tree_item(arc, tree_ref.lock().unwrap().first_selected_item().unwrap())
    });

    pack.add(&frame);
    pack.end();

    wind.make_resizable(true);
    wind.end();
    wind.show();
    app.run().unwrap();
}
