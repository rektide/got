use anyhow::Result;

pub fn get_untracked_files() -> Result<Vec<String>> {
    let repo = gixkit::open_repo(".")?;

    let files: Vec<String> = gixkit::UntrackedIterBuilder::new(&repo)
        .filter(gixkit::UntrackedFilter::Normal)
        .build()?
        .filter_map(|r| r.ok())
        .map(|status| status.path)
        .collect();

    Ok(files)
}

pub fn display_file_selection(files: &[String]) -> Result<()> {
    if files.is_empty() {
        println!("No untracked files found.");
        return Ok(());
    }

    println!("Select files to ignore (e.g., 1,3,5 or 1-5):");
    println!();

    for (i, file) in files.iter().enumerate() {
        println!("  [{}] {}", i + 1, file);
    }

    Ok(())
}

pub fn parse_selection(input: &str, max_index: usize) -> Vec<usize> {
    let mut selected = Vec::new();

    for part in input.split(',') {
        let part = part.trim();
        if part.contains('-') {
            let range: Vec<&str> = part.split('-').collect();
            if range.len() == 2 {
                if let (Ok(start), Ok(end)) = (range[0].parse::<usize>(), range[1].parse::<usize>()) {
                    let start = start.max(1);
                    let end = end.min(max_index);
                    for i in start..=end {
                        if !selected.contains(&i) {
                            selected.push(i);
                        }
                    }
                }
            }
        } else {
            if let Ok(n) = part.parse::<usize>() {
                if n >= 1 && n <= max_index && !selected.contains(&n) {
                    selected.push(n);
                }
            }
        }
    }

    selected.sort();
    selected
}

    };

    let index = repo.index().context("Failed to get index")?;

    for (path, _) in index.entries_with_paths_by_filter_map(|p, e| Some((p, e.id))) {
        let path_str = path.to_string();
        let path_iter = path.split(|&b| b == b'/');
        let mut lookup_buf = Vec::new();

        if head_tree
            .lookup_entry(path_iter, &mut lookup_buf)?
            .is_none()
        {
            files.push(path_str);
        }
    }

    Ok(files)
}

pub fn display_file_selection(files: &[String]) -> Result<()> {
    if files.is_empty() {
        println!("No untracked files found.");
        return Ok(());
    }

    println!("Select files to ignore (e.g., 1,3,5 or 1-5):");
    println!();

    for (i, file) in files.iter().enumerate() {
        println!("  [{}] {}", i + 1, file);
    }

    Ok(())
}

pub fn parse_selection(input: &str, max_index: usize) -> Vec<usize> {
    let mut selected = Vec::new();

    for part in input.split(',') {
        let part = part.trim();
        if part.contains('-') {
            let range: Vec<&str> = part.split('-').collect();
            if range.len() == 2 {
                if let (Ok(start), Ok(end)) = (range[0].parse::<usize>(), range[1].parse::<usize>())
                {
                    let start = start.max(1);
                    let end = end.min(max_index);
                    for i in start..=end {
                        if !selected.contains(&i) {
                            selected.push(i);
                        }
                    }
                }
            }
        } else {
            if let Ok(n) = part.parse::<usize>() {
                if n >= 1 && n <= max_index && !selected.contains(&n) {
                    selected.push(n);
                }
            }
        }
    }

    selected.sort();
    selected
}
