use fltk::{app, tree::{Tree, TreeItem}, dialog::{FileChooser, FileChooserType}};
use smash_arc::*;

use std::path::Path;

enum DirOrFile {
    Dir(Hash40),
    File(Hash40)
}

fn dir_info_print_children(arc: &ArcFile, dir_info: &DirInfo, data: &mut Vec<DirOrFile>) {
    let start = dir_info.child_dir_start_index as usize;
    let end = (dir_info.child_dir_start_index as usize) + (dir_info.child_dir_count as usize);

    let children = &arc.file_system.folder_child_hashes[start..end].iter()
        .map(|child| &arc.file_system.dir_infos[child.index() as usize])
        .collect::<Vec<_>>();

    for &child in children {
        data.push(DirOrFile::Dir(child.name));
    }

    dir_info_print_filepaths(arc, dir_info, data)
}

fn dir_info_print_filepaths(arc: &ArcFile, dir_info: &DirInfo, data: &mut Vec<DirOrFile>) {
    let file_infos = &arc.get_file_infos()[dir_info.file_info_range()].iter().collect::<Vec<_>>();

    for infos in file_infos {
        let file_path = arc.get_file_paths()[infos.file_path_index].path.hash40();
        let lbl = format!("{:#x}", file_path.as_u64());
        let res = format!("{}", file_path.global_label().unwrap_or(lbl));
        
        data.push(DirOrFile::File(Hash40::from(&res[..])));
    }

    if let Some(dep) = arc.get_directory_dependency(dir_info) {
        match dep {
            RedirectionType::Symlink(dir_info) => {
                dir_info_print_filepaths(arc, &dir_info, data);
            },
            RedirectionType::Shared(dir_offs) => {
                dir_offset_print_filepaths(arc, &dir_offs, data);
            },
        }
    };
}

fn dir_offset_print_filepaths(arc: &ArcFile, dir_info: &DirectoryOffset, data: &mut Vec<DirOrFile>) {
    let start = dir_info.file_start_index as usize;
    let end = (dir_info.file_start_index as usize) + (dir_info.file_count as usize);

    let file_infos = &arc.get_file_infos()[start..end].iter().collect::<Vec<_>>();

    for infos in file_infos {
        let file_path = arc.get_file_paths()[infos.file_path_index].path.hash40();
        let lbl = format!("{:#x}", file_path.as_u64());
        let res = format!("{}", file_path.global_label().unwrap_or(lbl));
        
        data.push(DirOrFile::File(Hash40::from(&res[..])));
    }
}

pub fn build_tree(arc: &ArcFile, tree: &mut Tree, path: &String, depth_left: usize) -> Result<(), ()> {
    let mut data: Vec<DirOrFile> = vec![];

    match arc.get_dir_info_from_hash(Hash40::from(&path[..])) {
        Ok(dir_info) => {
            dir_info_print_children(&arc, dir_info, &mut data);
        
            for node in data {
                match node {
                    DirOrFile::Dir(dir) => {
                        let dir_name = dir.global_label().unwrap();
                        let full_path = format!("{}/{}", path, dir_name);
                        match tree.add(&full_path) {
                            Some(mut res) => {
                                res.set_label_color(fltk::enums::Color::DarkRed)
                            },
                            None => {}
                        }
                        if depth_left > 0 {
                            build_tree(arc, tree, &full_path, depth_left - 1).unwrap();
                        }
                        let _ = tree.close(&full_path, false);
                    }
                    DirOrFile::File(file) => {
                        let file_name = file.global_label().unwrap_or(format!("{:#x}", file.as_u64()));
                        let full_path = format!("{}/{}", path, file_name);
                        tree.add(&full_path);
                    }
                }
            }
        },
        Err(_err) => {}
    }

    Ok(())
}

pub fn get_path(tree_item: TreeItem) -> String {
    if let Some(label) = tree_item.label() {
        if let Some(parent) = tree_item.parent() {
            let path = get_path(parent);
            if path == "/" {
                label
            } else {
                format!("{}/{}", path, label)
            }
        } else {
            label
        }
    } else {
        "".to_owned()
    }
}

pub fn extract_tree_item(arc: &ArcFile, tree_item: TreeItem) {
    let path = get_path(tree_item);
    let contents = arc.get_file_contents(&*path, smash_arc::Region::UsEnglish).unwrap();
    let path = Path::new(&path);
    let file_name = path.file_name().unwrap().to_string_lossy();

    let out = {
        let mut dialog = FileChooser::new(".", "*", FileChooserType::Create, "Extract File");
        dialog.set_value(&file_name);
        dialog.show();
        while dialog.shown() {
            app::wait();
        }

        dialog.value(1)
    };

    if let Some(path) = out {
        std::fs::write(
            path,
            contents
        ).unwrap();
    }
}
